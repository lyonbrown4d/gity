package runneragent

import (
	"bytes"
	"context"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"strings"

	cidomain "github.com/DaiYuANg/gity/internal/domain/ci"
	"github.com/samber/oops"
)

type Client struct {
	baseURL string
	token   string
	http    *http.Client
}

type ClaimResponse struct {
	Claimed bool                `json:"claimed"`
	Job     cidomain.ProjectJob `json:"job"`
}

type SourceArchiveResponse struct {
	FileName      string `json:"file_name"`
	Encoding      string `json:"encoding"`
	ContentBase64 string `json:"content_base64"`
}

func NewClient(baseURL, token string) *Client {
	return &Client{
		baseURL: strings.TrimRight(strings.TrimSpace(baseURL), "/"),
		token:   strings.TrimSpace(token),
		http:    http.DefaultClient,
	}
}

func (c *Client) Heartbeat(ctx context.Context) error {
	var out map[string]any
	return c.post(ctx, "/runners/heartbeat", map[string]any{"token": c.token}, &out)
}

func (c *Client) ClaimJob(ctx context.Context, leaseSeconds int) (ClaimResponse, error) {
	var out ClaimResponse
	err := c.post(ctx, "/runners/jobs/claim", map[string]any{
		"token":         c.token,
		"lease_seconds": leaseSeconds,
	}, &out)
	return out, err
}

func (c *Client) GetProjectJob(ctx context.Context, projectID, jobID int64) (cidomain.ProjectJob, error) {
	var out cidomain.ProjectJob
	err := c.get(ctx, fmt.Sprintf("/projects/%d/jobs/%d", projectID, jobID), &out)
	return out, err
}

func (c *Client) CompleteJob(ctx context.Context, jobID int64, result string) error {
	var out cidomain.ProjectJob
	return c.post(ctx, fmt.Sprintf("/runners/jobs/%d/complete", jobID), map[string]any{
		"token":  c.token,
		"result": result,
	}, &out)
}

func (c *Client) FailJob(ctx context.Context, jobID int64, message, result string, retryAfterSeconds int) error {
	var out cidomain.ProjectJob
	return c.post(ctx, fmt.Sprintf("/runners/jobs/%d/fail", jobID), map[string]any{
		"token":               c.token,
		"error":               message,
		"result":              result,
		"retry_after_seconds": retryAfterSeconds,
	}, &out)
}

func (c *Client) AppendTrace(ctx context.Context, jobID int64, output string, outputTruncated bool, durationMillis int64) error {
	var out map[string]any
	return c.post(ctx, fmt.Sprintf("/runners/jobs/%d/trace", jobID), map[string]any{
		"token":            c.token,
		"output":           output,
		"output_truncated": outputTruncated,
		"duration_millis":  durationMillis,
	}, &out)
}

func (c *Client) DownloadSourceArchive(ctx context.Context, jobID int64) ([]byte, error) {
	var out SourceArchiveResponse
	if err := c.post(ctx, fmt.Sprintf("/runners/jobs/%d/source-archive", jobID), map[string]any{
		"token": c.token,
	}, &out); err != nil {
		return nil, oops.In("runner_agent").With("job_id", jobID).Wrapf(err, "request source archive")
	}
	if strings.TrimSpace(out.Encoding) != "" && strings.TrimSpace(out.Encoding) != "base64" {
		return nil, oops.In("runner_agent").With("job_id", jobID, "encoding", out.Encoding).New("unsupported source archive encoding")
	}
	content, err := base64.StdEncoding.DecodeString(strings.TrimSpace(out.ContentBase64))
	if err != nil {
		return nil, oops.In("runner_agent").With("job_id", jobID).Wrapf(err, "decode source archive")
	}
	return content, nil
}

func (c *Client) UploadArtifact(ctx context.Context, jobID int64, artifact ArtifactFile) error {
	var out cidomain.ProjectJobArtifact
	return c.post(ctx, fmt.Sprintf("/runners/jobs/%d/artifacts", jobID), map[string]any{
		"token":          c.token,
		"name":           artifact.Name,
		"file_name":      artifact.FileName,
		"file_path":      artifact.FilePath,
		"content_type":   artifact.ContentType,
		"content_base64": base64.StdEncoding.EncodeToString(artifact.Content),
	}, &out)
}

func (c *Client) get(ctx context.Context, path string, out any) (err error) {
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, c.baseURL+path, http.NoBody)
	if err != nil {
		return oops.In("runner_agent").With("method", http.MethodGet, "path", path).Wrapf(err, "create runner request")
	}
	return c.do(req, http.MethodGet, path, out)
}

func (c *Client) post(ctx context.Context, path string, payload, out any) (err error) {
	body, err := json.Marshal(payload)
	if err != nil {
		return oops.In("runner_agent").With("method", http.MethodPost, "path", path).Wrapf(err, "encode runner request")
	}
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, c.baseURL+path, bytes.NewReader(body))
	if err != nil {
		return oops.In("runner_agent").With("method", http.MethodPost, "path", path).Wrapf(err, "create runner request")
	}
	req.Header.Set("Content-Type", "application/json")
	return c.do(req, http.MethodPost, path, out)
}

func (c *Client) do(req *http.Request, method, path string, out any) (err error) {
	resp, err := c.http.Do(req)
	if err != nil {
		return oops.In("runner_agent").With("method", method, "path", path).Wrapf(err, "send runner request")
	}
	defer func() {
		if closeErr := resp.Body.Close(); closeErr != nil {
			if err != nil {
				err = oops.In("runner_agent").With("path", path).Wrapf(oops.Join(err, closeErr), "runner request and close response body")
				return
			}
			err = oops.In("runner_agent").With("path", path).Wrapf(closeErr, "close runner response body")
		}
	}()
	return c.handleResponse(resp, method, path, out)
}

func (c *Client) handleResponse(resp *http.Response, method, path string, out any) error {
	content, err := io.ReadAll(resp.Body)
	if err != nil {
		return oops.In("runner_agent").With("method", method, "path", path).Wrapf(err, "read runner response")
	}
	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		return oops.In("runner_agent").With("method", method, "path", path, "status", resp.StatusCode, "body", strings.TrimSpace(string(content))).New("runner request failed")
	}
	if out == nil {
		return nil
	}
	if err := decodeBody(content, out); err != nil {
		return oops.In("runner_agent").With("method", method, "path", path).Wrapf(err, "decode runner response")
	}
	return nil
}

func decodeBody(content []byte, out any) error {
	var wrapper struct {
		Body json.RawMessage `json:"body"`
	}
	if err := json.Unmarshal(content, &wrapper); err != nil {
		return oops.In("runner_agent").Wrapf(err, "decode response wrapper")
	}
	if len(wrapper.Body) == 0 {
		if err := json.Unmarshal(content, out); err != nil {
			return oops.In("runner_agent").Wrapf(err, "decode raw response body")
		}
		return nil
	}
	if err := json.Unmarshal(wrapper.Body, out); err != nil {
		return oops.In("runner_agent").Wrapf(err, "decode wrapped response body")
	}
	return nil
}
