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

	"github.com/DaiYuANg/gity/internal/entity"
)

type Client struct {
	baseURL string
	token   string
	http    *http.Client
}

type ClaimResponse struct {
	Claimed bool              `json:"claimed"`
	Job     entity.ProjectJob `json:"job"`
}

func NewClient(baseURL string, token string) *Client {
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

func (c *Client) GetProjectJob(ctx context.Context, projectID int64, jobID int64) (entity.ProjectJob, error) {
	var out entity.ProjectJob
	err := c.get(ctx, fmt.Sprintf("/projects/%d/jobs/%d", projectID, jobID), &out)
	return out, err
}

func (c *Client) CompleteJob(ctx context.Context, jobID int64, result string) error {
	var out entity.ProjectJob
	return c.post(ctx, fmt.Sprintf("/runners/jobs/%d/complete", jobID), map[string]any{
		"token":  c.token,
		"result": result,
	}, &out)
}

func (c *Client) FailJob(ctx context.Context, jobID int64, message string, result string, retryAfterSeconds int) error {
	var out entity.ProjectJob
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

func (c *Client) UploadArtifact(ctx context.Context, jobID int64, artifact ArtifactFile) error {
	var out entity.ProjectJobArtifact
	return c.post(ctx, fmt.Sprintf("/runners/jobs/%d/artifacts", jobID), map[string]any{
		"token":          c.token,
		"name":           artifact.Name,
		"file_name":      artifact.FileName,
		"file_path":      artifact.FilePath,
		"content_type":   artifact.ContentType,
		"content_base64": base64.StdEncoding.EncodeToString(artifact.Content),
	}, &out)
}

func (c *Client) get(ctx context.Context, path string, out any) error {
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, c.baseURL+path, http.NoBody)
	if err != nil {
		return fmt.Errorf("create runner request: %w", err)
	}
	resp, err := c.http.Do(req)
	if err != nil {
		return fmt.Errorf("send runner request: %w", err)
	}
	defer resp.Body.Close()
	content, err := io.ReadAll(resp.Body)
	if err != nil {
		return fmt.Errorf("read runner response: %w", err)
	}
	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		return fmt.Errorf("runner request failed: status=%d body=%s", resp.StatusCode, strings.TrimSpace(string(content)))
	}
	if out == nil {
		return nil
	}
	if err := decodeBody(content, out); err != nil {
		return fmt.Errorf("decode runner response: %w", err)
	}
	return nil
}

func (c *Client) post(ctx context.Context, path string, payload any, out any) error {
	body, err := json.Marshal(payload)
	if err != nil {
		return fmt.Errorf("encode runner request: %w", err)
	}
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, c.baseURL+path, bytes.NewReader(body))
	if err != nil {
		return fmt.Errorf("create runner request: %w", err)
	}
	req.Header.Set("Content-Type", "application/json")
	resp, err := c.http.Do(req)
	if err != nil {
		return fmt.Errorf("send runner request: %w", err)
	}
	defer resp.Body.Close()
	content, err := io.ReadAll(resp.Body)
	if err != nil {
		return fmt.Errorf("read runner response: %w", err)
	}
	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		return fmt.Errorf("runner request failed: status=%d body=%s", resp.StatusCode, strings.TrimSpace(string(content)))
	}
	if out == nil {
		return nil
	}
	if err := decodeBody(content, out); err != nil {
		return fmt.Errorf("decode runner response: %w", err)
	}
	return nil
}

func decodeBody(content []byte, out any) error {
	var wrapper struct {
		Body json.RawMessage `json:"body"`
	}
	if err := json.Unmarshal(content, &wrapper); err != nil {
		return err
	}
	if len(wrapper.Body) == 0 {
		return json.Unmarshal(content, out)
	}
	return json.Unmarshal(wrapper.Body, out)
}
