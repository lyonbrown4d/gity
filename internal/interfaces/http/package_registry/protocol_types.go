package packageregistry

import "github.com/arcgolabs/httpx"

type protocolPackageFileInput struct {
	ProjectID      int64          `path:"id"`
	PackageName    string         `path:"package_name"`
	PackageVersion string         `path:"package_version"`
	FileName       httpx.PathTail `path:"file_name"`
	ContentType    string         `header:"Content-Type"`
	Authorization  string         `header:"Authorization"`
	Payload        httpx.RequestStream
}

type protocolPackageDownloadInput struct {
	ProjectID      int64          `path:"id"`
	PackageName    string         `path:"package_name"`
	PackageVersion string         `path:"package_version"`
	FileName       httpx.PathTail `path:"file_name"`
	Authorization  string         `header:"Authorization"`
}

type mavenPackageFileInput struct {
	ProjectID     int64          `path:"id"`
	FilePath      httpx.PathTail `path:"file_path"`
	ContentType   string         `header:"Content-Type"`
	Authorization string         `header:"Authorization"`
	Payload       httpx.RequestStream
}

type mavenPackageDownloadInput struct {
	ProjectID     int64          `path:"id"`
	FilePath      httpx.PathTail `path:"file_path"`
	Authorization string         `header:"Authorization"`
}

type npmPackageInput struct {
	ProjectID     int64          `path:"id"`
	PackageName   httpx.PathTail `path:"package_name"`
	Authorization string         `header:"Authorization"`
}

type npmPublishInput struct {
	ProjectID     int64          `path:"id"`
	PackageName   httpx.PathTail `path:"package_name"`
	Authorization string         `header:"Authorization"`
	Body          npmPublishBody `json:"body"`
}

type pypiIndexInput struct {
	ProjectID     int64  `path:"id"`
	Authorization string `header:"Authorization"`
}

type pypiPackageInput struct {
	ProjectID     int64  `path:"id"`
	PackageName   string `path:"package_name"`
	Authorization string `header:"Authorization"`
}

type packageFileDownloadInput struct {
	ProjectID     int64  `path:"id"`
	FileID        int64  `path:"file_id"`
	Authorization string `header:"Authorization"`
}

type packageBinaryOutput struct {
	ContentType        string `header:"Content-Type"`
	ContentDisposition string `header:"Content-Disposition"`
	Body               httpx.ResponseStream
}

type packageHTMLOutput struct {
	ContentType string `header:"Content-Type"`
	Body        httpx.ResponseStream
}

type npmPublishBody struct {
	Name        string                       `json:"name"`
	DistTags    map[string]string            `json:"dist-tags"`
	Versions    map[string]npmPublishVersion `json:"versions"`
	Attachments map[string]npmAttachment     `json:"_attachments"`
}

type npmPublishVersion struct {
	Name    string            `json:"name"`
	Version string            `json:"version"`
	Dist    map[string]string `json:"dist"`
}

type npmAttachment struct {
	ContentType string `json:"content_type"`
	Data        string `json:"data"`
	Length      int64  `json:"length"`
}

func (in protocolPackageFileInput) AuthorizationHeader() string { return in.Authorization }
func (in protocolPackageFileInput) ProjectIDValue() int64       { return in.ProjectID }
func (in protocolPackageDownloadInput) AuthorizationHeader() string {
	return in.Authorization
}
func (in protocolPackageDownloadInput) ProjectIDValue() int64 { return in.ProjectID }
func (in mavenPackageFileInput) AuthorizationHeader() string  { return in.Authorization }
func (in mavenPackageFileInput) ProjectIDValue() int64        { return in.ProjectID }
func (in mavenPackageDownloadInput) AuthorizationHeader() string {
	return in.Authorization
}
func (in mavenPackageDownloadInput) ProjectIDValue() int64 { return in.ProjectID }
func (in npmPackageInput) AuthorizationHeader() string     { return in.Authorization }
func (in npmPackageInput) ProjectIDValue() int64           { return in.ProjectID }
func (in npmPublishInput) AuthorizationHeader() string     { return in.Authorization }
func (in npmPublishInput) ProjectIDValue() int64           { return in.ProjectID }
func (in pypiIndexInput) AuthorizationHeader() string      { return in.Authorization }
func (in pypiIndexInput) ProjectIDValue() int64            { return in.ProjectID }
func (in pypiPackageInput) AuthorizationHeader() string    { return in.Authorization }
func (in pypiPackageInput) ProjectIDValue() int64          { return in.ProjectID }
func (in packageFileDownloadInput) AuthorizationHeader() string {
	return in.Authorization
}
func (in packageFileDownloadInput) ProjectIDValue() int64 { return in.ProjectID }
