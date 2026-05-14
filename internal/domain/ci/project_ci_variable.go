package ci

import "time"

type ProjectCIVariable struct {
	ID        int64     `dbx:"id"         json:"id"`
	ProjectID int64     `dbx:"project_id" json:"project_id"`
	Key       string    `dbx:"key"        json:"key"`
	Value     string    `dbx:"value"      json:"value,omitempty"`
	Masked    int       `dbx:"masked"     json:"masked"`
	Protected int       `dbx:"protected"  json:"protected"`
	CreatedAt time.Time `dbx:"created_at" json:"created_at"`
	UpdatedAt time.Time `dbx:"updated_at" json:"updated_at"`
}

func (v ProjectCIVariable) IsMasked() bool {
	return v.Masked != 0
}

func (v ProjectCIVariable) IsProtected() bool {
	return v.Protected != 0
}
