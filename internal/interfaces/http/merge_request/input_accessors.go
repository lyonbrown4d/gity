package mergerequest

func (in mergeRequestsInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in mergeRequestsInput) ProjectIDValue() int64 {
	return in.ProjectID
}

func (in mergeRequestInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in mergeRequestInput) ProjectIDValue() int64 {
	return in.ProjectID
}

func (in createMergeRequestInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in createMergeRequestInput) ProjectIDValue() int64 {
	return in.ProjectID
}

func (in updateMergeRequestInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in updateMergeRequestInput) ProjectIDValue() int64 {
	return in.ProjectID
}

func (in mergeMergeRequestInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in mergeMergeRequestInput) ProjectIDValue() int64 {
	return in.ProjectID
}

func (in createMergeRequestCommentInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in createMergeRequestCommentInput) ProjectIDValue() int64 {
	return in.ProjectID
}

func (in mergeRequestApprovalInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in mergeRequestApprovalInput) ProjectIDValue() int64 {
	return in.ProjectID
}

func (in approvalRulesInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in approvalRulesInput) ProjectIDValue() int64 {
	return in.ProjectID
}

func (in approvalRuleInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in approvalRuleInput) ProjectIDValue() int64 {
	return in.ProjectID
}

func (in createApprovalRuleInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in createApprovalRuleInput) ProjectIDValue() int64 {
	return in.ProjectID
}

func (in updateApprovalRuleInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in updateApprovalRuleInput) ProjectIDValue() int64 {
	return in.ProjectID
}

func (in setParticipantsInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in setParticipantsInput) ProjectIDValue() int64 {
	return in.ProjectID
}
