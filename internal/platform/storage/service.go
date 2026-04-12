package storage

import (
	"bytes"
	"context"
	"fmt"
	"io"
	"os"
	"path"
	"path/filepath"
	"strings"

	"github.com/DaiYuANg/gity/internal/config"
	"github.com/aws/aws-sdk-go-v2/aws"
	awsconfig "github.com/aws/aws-sdk-go-v2/config"
	"github.com/aws/aws-sdk-go-v2/credentials"
	"github.com/aws/aws-sdk-go-v2/service/s3"
	"github.com/aws/aws-sdk-go-v2/service/s3/types"
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
			return nil, err
		}
		return &Service{backend: client}, nil
	default:
		return nil, fmt.Errorf("unsupported storage driver: %s", driver)
	}
}

func (s *Service) SaveObject(ctx context.Context, key string, content []byte, contentType string) error {
	if s == nil || s.backend == nil {
		return fmt.Errorf("storage backend is not configured")
	}
	return s.backend.SaveObject(ctx, key, content, contentType)
}

func (s *Service) SaveIssueAttachment(ctx context.Context, projectFullPath string, issueIID int64, attachmentID int64, fileName string, content []byte, contentType string) (string, error) {
	key := BuildIssueStorageKey(projectFullPath, issueIID, attachmentID, fileName)
	if err := s.SaveObject(ctx, key, content, contentType); err != nil {
		return "", err
	}
	return key, nil
}

func (s *Service) SavePackageFile(ctx context.Context, projectFullPath string, packageType string, packageName string, version string, fileID int64, fileName string, content []byte, contentType string) (string, error) {
	key := BuildPackageStorageKey(projectFullPath, packageType, packageName, version, fileID, fileName)
	if err := s.SaveObject(ctx, key, content, contentType); err != nil {
		return "", err
	}
	return key, nil
}

func (s *Service) SaveLFSObject(ctx context.Context, projectFullPath string, oid string, content []byte) (string, error) {
	key := BuildLFSStorageKey(projectFullPath, oid)
	if err := s.SaveObject(ctx, key, content, "application/octet-stream"); err != nil {
		return "", err
	}
	return key, nil
}

func (s *Service) Load(ctx context.Context, key string) ([]byte, error) {
	if s == nil || s.backend == nil {
		return nil, fmt.Errorf("storage backend is not configured")
	}
	return s.backend.Load(ctx, key)
}

type localBackend struct {
	root string
}

func (s *localBackend) SaveObject(_ context.Context, key string, content []byte, _ string) error {
	absPath, err := s.resolveKey(key)
	if err != nil {
		return err
	}
	if err := os.MkdirAll(filepath.Dir(absPath), 0o755); err != nil {
		return fmt.Errorf("create object directory: %w", err)
	}
	if err := os.WriteFile(absPath, content, 0o644); err != nil {
		return fmt.Errorf("write object file: %w", err)
	}
	return nil
}

func (s *localBackend) Load(_ context.Context, key string) ([]byte, error) {
	absPath, err := s.resolveKey(key)
	if err != nil {
		return nil, err
	}
	content, err := os.ReadFile(absPath)
	if err != nil {
		return nil, fmt.Errorf("read object file: %w", err)
	}
	return content, nil
}

func (s *localBackend) resolveKey(key string) (string, error) {
	root, err := filepath.Abs(s.root)
	if err != nil {
		return "", fmt.Errorf("resolve storage root: %w", err)
	}
	relative := filepath.Clean(filepath.FromSlash(strings.Trim(strings.ReplaceAll(key, "\\", "/"), "/")))
	if relative == "." || strings.HasPrefix(relative, "..") {
		return "", fmt.Errorf("invalid storage key")
	}
	absPath, err := filepath.Abs(filepath.Join(root, relative))
	if err != nil {
		return "", fmt.Errorf("resolve storage path: %w", err)
	}
	if absPath != root && !strings.HasPrefix(absPath, root+string(filepath.Separator)) {
		return "", fmt.Errorf("storage key escapes storage root")
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
		return nil, fmt.Errorf("storage s3 bucket is required when storage driver is s3")
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
		return nil, fmt.Errorf("load s3 config: %w", err)
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
		return fmt.Errorf("put s3 object: %w", err)
	}
	return nil
}

func (s *s3Backend) Load(ctx context.Context, key string) ([]byte, error) {
	result, err := s.client.GetObject(ctx, &s3.GetObjectInput{Bucket: aws.String(s.bucket), Key: aws.String(normalizeStorageKey(key))})
	if err != nil {
		return nil, fmt.Errorf("get s3 object: %w", err)
	}
	defer result.Body.Close()
	content, err := io.ReadAll(result.Body)
	if err != nil {
		return nil, fmt.Errorf("read s3 object body: %w", err)
	}
	return content, nil
}

func (s *s3Backend) ensureBucket(ctx context.Context) error {
	_, err := s.client.HeadBucket(ctx, &s3.HeadBucketInput{Bucket: aws.String(s.bucket)})
	if err == nil {
		return nil
	}
	if !s.autoCreateBucket {
		return fmt.Errorf("check s3 bucket %s: %w", s.bucket, err)
	}
	_, createErr := s.client.CreateBucket(ctx, &s3.CreateBucketInput{Bucket: aws.String(s.bucket)})
	if createErr != nil {
		return fmt.Errorf("create s3 bucket %s: %w", s.bucket, createErr)
	}
	return nil
}

func BuildIssueStorageKey(projectFullPath string, issueIID int64, attachmentID int64, fileName string) string {
	return path.Join("issues", sanitizeNestedPath(projectFullPath), fmt.Sprintf("%d", issueIID), fmt.Sprintf("%d", attachmentID), sanitizeFileName(fileName))
}

func BuildPackageStorageKey(projectFullPath string, packageType string, packageName string, version string, fileID int64, fileName string) string {
	return path.Join("packages", sanitizeNestedPath(projectFullPath), sanitizePathSegment(packageType), sanitizePathSegment(packageName), sanitizePathSegment(version), fmt.Sprintf("%d", fileID), sanitizeFileName(fileName))
}

func BuildLFSStorageKey(projectFullPath string, oid string) string {
	return path.Join("lfs", sanitizeNestedPath(projectFullPath), sanitizePathSegment(oid))
}

func DetectContentType(fileName string) string {
	name := strings.ToLower(strings.TrimSpace(fileName))
	switch {
	case strings.HasSuffix(name, ".md"):
		return "text/markdown"
	case strings.HasSuffix(name, ".txt"):
		return "text/plain"
	case strings.HasSuffix(name, ".json"):
		return "application/json"
	case strings.HasSuffix(name, ".xml"):
		return "application/xml"
	case strings.HasSuffix(name, ".pom"):
		return "application/xml"
	case strings.HasSuffix(name, ".jar"):
		return "application/java-archive"
	case strings.HasSuffix(name, ".tgz"):
		return "application/gzip"
	case strings.HasSuffix(name, ".tar"):
		return "application/x-tar"
	case strings.HasSuffix(name, ".png"):
		return "image/png"
	case strings.HasSuffix(name, ".jpg") || strings.HasSuffix(name, ".jpeg"):
		return "image/jpeg"
	case strings.HasSuffix(name, ".gif"):
		return "image/gif"
	default:
		return "application/octet-stream"
	}
}

func normalizeStorageKey(key string) string {
	return strings.Trim(strings.ReplaceAll(key, "\\", "/"), "/")
}

func sanitizeNestedPath(value string) string {
	parts := strings.Split(normalizeStorageKey(value), "/")
	sanitized := make([]string, 0, len(parts))
	for _, part := range parts {
		part = sanitizePathSegment(part)
		if part == "" {
			continue
		}
		sanitized = append(sanitized, part)
	}
	if len(sanitized) == 0 {
		return "unknown"
	}
	return path.Join(sanitized...)
}

func sanitizePathSegment(value string) string {
	trimmed := strings.TrimSpace(value)
	if trimmed == "" {
		return "unknown"
	}
	replacer := strings.NewReplacer("<", "_", ">", "_", ":", "_", "\"", "_", "|", "_", "?", "_", "*", "_", "\\", "_", "/", "_")
	trimmed = replacer.Replace(trimmed)
	trimmed = strings.Trim(trimmed, ". ")
	if trimmed == "" {
		return "unknown"
	}
	return trimmed
}

func sanitizeFileName(fileName string) string {
	trimmed := strings.TrimSpace(fileName)
	if trimmed == "" {
		return "blob.bin"
	}
	trimmed = strings.ReplaceAll(trimmed, "\\", "/")
	trimmed = path.Base(trimmed)
	trimmed = sanitizePathSegment(trimmed)
	if trimmed == "" || trimmed == "unknown" {
		return "blob.bin"
	}
	return trimmed
}

var _ types.BucketLocationConstraint
