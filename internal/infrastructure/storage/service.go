package storage

import (
	"bytes"
	"context"
	"io"
	"os"
	"path/filepath"
	"strings"

	storageports "github.com/DaiYuANg/gity/internal/application/ports"
	"github.com/DaiYuANg/gity/internal/config"
	"github.com/aws/aws-sdk-go-v2/aws"
	awsconfig "github.com/aws/aws-sdk-go-v2/config"
	"github.com/aws/aws-sdk-go-v2/credentials"
	"github.com/aws/aws-sdk-go-v2/service/s3"
	"github.com/aws/aws-sdk-go-v2/service/s3/types"
	"github.com/samber/oops"
)

type backend interface {
	SaveObject(ctx context.Context, key string, content []byte, contentType string) error
	Load(ctx context.Context, key string) ([]byte, error)
}

type Service struct {
	backend backend
}

func NewService(settings config.Settings) (*Service, error) {
	driver := strings.TrimSpace(strings.ToLower(settings.Storage.Driver))
	if driver == "" {
		driver = "local"
	}
	switch driver {
	case "local":
		return &Service{backend: &localBackend{root: settings.Storage.Root}}, nil
	case "s3":
		client, err := newS3Backend(settings)
		if err != nil {
			return nil, oops.In("storage").With("driver", driver).Wrapf(err, "create storage backend")
		}
		return &Service{backend: client}, nil
	default:
		return nil, oops.In("storage").With("driver", driver).New("unsupported storage driver")
	}
}

func (s *Service) SaveObject(ctx context.Context, key string, content []byte, contentType string) error {
	if s == nil || s.backend == nil {
		return oops.In("storage").With("key", key).New("storage backend is not configured")
	}
	if err := s.backend.SaveObject(ctx, key, content, contentType); err != nil {
		return oops.In("storage").With("key", key, "content_type", contentType, "byte_size", len(content)).Wrapf(err, "save object")
	}
	return nil
}

func (s *Service) SaveIssueAttachment(ctx context.Context, projectFullPath string, issueIID int64, attachmentID int64, fileName string, content []byte, contentType string) (string, error) {
	key := storageports.BuildIssueStorageKey(projectFullPath, issueIID, attachmentID, fileName)
	if err := s.SaveObject(ctx, key, content, contentType); err != nil {
		return "", oops.In("storage").With("project", projectFullPath, "issue_iid", issueIID, "attachment_id", attachmentID).Wrapf(err, "save issue attachment")
	}
	return key, nil
}

func (s *Service) SavePackageFile(ctx context.Context, projectFullPath string, packageType string, packageName string, version string, fileID int64, fileName string, content []byte, contentType string) (string, error) {
	key := storageports.BuildPackageStorageKey(projectFullPath, packageType, packageName, version, fileID, fileName)
	if err := s.SaveObject(ctx, key, content, contentType); err != nil {
		return "", oops.In("storage").With("project", projectFullPath, "package_type", packageType, "package_name", packageName, "version", version, "file_id", fileID).Wrapf(err, "save package file")
	}
	return key, nil
}

func (s *Service) SaveLFSObject(ctx context.Context, projectFullPath string, oid string, content []byte) (string, error) {
	key := storageports.BuildLFSStorageKey(projectFullPath, oid)
	if err := s.SaveObject(ctx, key, content, "application/octet-stream"); err != nil {
		return "", oops.In("storage").With("project", projectFullPath, "oid", oid).Wrapf(err, "save lfs object")
	}
	return key, nil
}

func (s *Service) SavePipelineArtifact(ctx context.Context, projectFullPath string, projectJobID int64, artifactID int64, fileName string, content []byte, contentType string) (string, error) {
	key := storageports.BuildPipelineArtifactStorageKey(projectFullPath, projectJobID, artifactID, fileName)
	if err := s.SaveObject(ctx, key, content, contentType); err != nil {
		return "", oops.In("storage").With("project", projectFullPath, "project_job_id", projectJobID, "artifact_id", artifactID).Wrapf(err, "save pipeline artifact")
	}
	return key, nil
}

func (s *Service) Load(ctx context.Context, key string) ([]byte, error) {
	if s == nil || s.backend == nil {
		return nil, oops.In("storage").With("key", key).New("storage backend is not configured")
	}
	content, err := s.backend.Load(ctx, key)
	if err != nil {
		return nil, oops.In("storage").With("key", key).Wrapf(err, "load object")
	}
	return content, nil
}

type localBackend struct {
	root string
}

func (s *localBackend) SaveObject(_ context.Context, key string, content []byte, _ string) error {
	absPath, err := s.resolveKey(key)
	if err != nil {
		return oops.In("storage").With("key", key).Wrapf(err, "resolve local object key")
	}
	if err := os.MkdirAll(filepath.Dir(absPath), 0o755); err != nil {
		return oops.In("storage").With("path", filepath.Dir(absPath)).Wrapf(err, "create object directory")
	}
	if err := os.WriteFile(absPath, content, 0o644); err != nil {
		return oops.In("storage").With("path", absPath, "byte_size", len(content)).Wrapf(err, "write object file")
	}
	return nil
}

func (s *localBackend) Load(_ context.Context, key string) ([]byte, error) {
	absPath, err := s.resolveKey(key)
	if err != nil {
		return nil, oops.In("storage").With("key", key).Wrapf(err, "resolve local object key")
	}
	content, err := os.ReadFile(absPath)
	if err != nil {
		return nil, oops.In("storage").With("path", absPath).Wrapf(err, "read object file")
	}
	return content, nil
}

func (s *localBackend) resolveKey(key string) (string, error) {
	root, err := filepath.Abs(s.root)
	if err != nil {
		return "", oops.In("storage").With("root", s.root).Wrapf(err, "resolve storage root")
	}
	relative := filepath.Clean(filepath.FromSlash(strings.Trim(strings.ReplaceAll(key, "\\", "/"), "/")))
	if relative == "." || strings.HasPrefix(relative, "..") {
		return "", oops.In("storage").With("key", key).New("invalid storage key")
	}
	absPath, err := filepath.Abs(filepath.Join(root, relative))
	if err != nil {
		return "", oops.In("storage").With("root", root, "relative", relative).Wrapf(err, "resolve storage path")
	}
	if absPath != root && !strings.HasPrefix(absPath, root+string(filepath.Separator)) {
		return "", oops.In("storage").With("root", root, "path", absPath, "key", key).New("storage key escapes storage root")
	}
	return absPath, nil
}

type s3Backend struct {
	client           *s3.Client
	bucket           string
	autoCreateBucket bool
}

func newS3Backend(settings config.Settings) (*s3Backend, error) {
	bucket := strings.TrimSpace(settings.Storage.S3Bucket)
	if bucket == "" {
		return nil, oops.In("storage").New("storage s3 bucket is required when storage driver is s3")
	}
	loadOptions := make([]func(*awsconfig.LoadOptions) error, 0, 2)
	if strings.TrimSpace(settings.Storage.S3Region) != "" {
		loadOptions = append(loadOptions, awsconfig.WithRegion(strings.TrimSpace(settings.Storage.S3Region)))
	}
	if strings.TrimSpace(settings.Storage.S3AccessKey) != "" || strings.TrimSpace(settings.Storage.S3SecretKey) != "" {
		loadOptions = append(loadOptions, awsconfig.WithCredentialsProvider(credentials.NewStaticCredentialsProvider(
			strings.TrimSpace(settings.Storage.S3AccessKey),
			strings.TrimSpace(settings.Storage.S3SecretKey),
			"",
		)))
	}
	cfg, err := awsconfig.LoadDefaultConfig(context.Background(), loadOptions...)
	if err != nil {
		return nil, oops.In("storage").With("bucket", bucket).Wrapf(err, "load s3 config")
	}
	client := s3.NewFromConfig(cfg, func(o *s3.Options) {
		o.UsePathStyle = settings.Storage.S3UsePathStyle
		if endpoint := strings.TrimSpace(settings.Storage.S3Endpoint); endpoint != "" {
			o.BaseEndpoint = aws.String(endpoint)
		}
	})
	return &s3Backend{client: client, bucket: bucket, autoCreateBucket: settings.Storage.S3AutoCreateBucket}, nil
}

func (s *s3Backend) SaveObject(ctx context.Context, key string, content []byte, contentType string) error {
	if err := s.ensureBucket(ctx); err != nil {
		return err
	}
	if strings.TrimSpace(contentType) == "" {
		contentType = "application/octet-stream"
	}
	_, err := s.client.PutObject(ctx, &s3.PutObjectInput{Bucket: aws.String(s.bucket), Key: aws.String(normalizeStorageKey(key)), Body: bytes.NewReader(content), ContentType: aws.String(contentType)})
	if err != nil {
		return oops.In("storage").With("bucket", s.bucket, "key", key, "content_type", contentType, "byte_size", len(content)).Wrapf(err, "put s3 object")
	}
	return nil
}

func (s *s3Backend) Load(ctx context.Context, key string) (content []byte, err error) {
	result, err := s.client.GetObject(ctx, &s3.GetObjectInput{Bucket: aws.String(s.bucket), Key: aws.String(normalizeStorageKey(key))})
	if err != nil {
		return nil, oops.In("storage").With("bucket", s.bucket, "key", key).Wrapf(err, "get s3 object")
	}
	defer func() {
		if closeErr := result.Body.Close(); closeErr != nil {
			if err != nil {
				err = oops.In("storage").With("bucket", s.bucket, "key", key).Wrapf(oops.Join(err, closeErr), "read s3 object body and close response")
				return
			}
			err = oops.In("storage").With("bucket", s.bucket, "key", key).Wrapf(closeErr, "close s3 object body")
		}
	}()
	content, err = io.ReadAll(result.Body)
	if err != nil {
		return nil, oops.In("storage").With("bucket", s.bucket, "key", key).Wrapf(err, "read s3 object body")
	}
	return content, nil
}

func (s *s3Backend) ensureBucket(ctx context.Context) error {
	_, err := s.client.HeadBucket(ctx, &s3.HeadBucketInput{Bucket: aws.String(s.bucket)})
	if err == nil {
		return nil
	}
	if !s.autoCreateBucket {
		return oops.In("storage").With("bucket", s.bucket).Wrapf(err, "check s3 bucket")
	}
	_, createErr := s.client.CreateBucket(ctx, &s3.CreateBucketInput{Bucket: aws.String(s.bucket)})
	if createErr != nil {
		return oops.In("storage").With("bucket", s.bucket).Wrapf(createErr, "create s3 bucket")
	}
	return nil
}

func normalizeStorageKey(key string) string {
	return strings.Trim(strings.ReplaceAll(key, "\\", "/"), "/")
}

var _ types.BucketLocationConstraint
