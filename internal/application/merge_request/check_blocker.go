package mergerequest

const (
	mergeCheckBlockerCategoryApproval = "approval"
	mergeCheckBlockerCategoryPipeline = "pipeline"

	mergeCheckBlockerApprovalRuleUnsatisfied = "approval_rule_unsatisfied"
	mergeCheckBlockerPipelineMissing         = "pipeline_missing"
	mergeCheckBlockerPipelineNotSucceeded    = "pipeline_not_succeeded"
	mergeCheckBlockerPipelineRepoMissing     = "pipeline_repository_missing"
)

func addMergeCheckBlocker(view CheckStatusView, blocker CheckBlockerView) CheckStatusView {
	if blocker.Message == "" {
		return view
	}
	view.Blockers = append(view.Blockers, blocker)
	if view.BlockingReason == "" {
		view.BlockingReason = blocker.Message
	}
	return view
}

func blockMergeCheck(view CheckStatusView, blocker CheckBlockerView) CheckStatusView {
	view.Mergeable = false
	previousReason := view.BlockingReason
	view = addMergeCheckBlocker(view, blocker)
	if previousReason == "" && view.BlockingReason != "" {
		view.Status = "blocked"
	}
	return view
}

func passMergeCheck(view CheckStatusView) CheckStatusView {
	if view.BlockingReason != "" {
		return view
	}
	view.Mergeable = true
	if view.Status == "not_required" {
		view.Status = "passed"
	}
	return view
}
