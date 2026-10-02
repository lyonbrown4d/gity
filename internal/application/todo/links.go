package todo

import (
	"fmt"
	"strconv"
)

func projectURL(organizationID, projectID int64) string {
	return fmt.Sprintf("/app/projects/%d/%d", organizationID, projectID)
}

func issueURL(organizationID, projectID, issueIID int64) string {
	return fmt.Sprintf("/app/projects/%d/%d/issues/%d", organizationID, projectID, issueIID)
}

func formatID(id int64) string {
	return strconv.FormatInt(id, 10)
}
