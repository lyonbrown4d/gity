package plandsl_test

import (
	"context"
	"testing"

	plandsl "github.com/lyonbrown4d/gity/internal/ci/plan_dsl"
)

func TestCompilePipelineDSL(t *testing.T) {
	t.Parallel()

	spec, err := plandsl.Compile(context.Background(), "ci.plano", `
fn image(name: string): string {
  return name + ":latest"
}

pipeline {
  name = "release"
}

stage lint {
  image = image("golangci")
  timeout_seconds = 120
  run {
    shell("golangci-lint run ./...")
  }
}

stage test {
  needs = [lint]
  image = "golang:1.26"
  artifacts = ["coverage.out"]
  run {
    exec("go", "test", "./...")
  }
}
`)
	if err != nil {
		t.Fatalf("compile dsl: %v", err)
	}
	assertPipelineSpec(t, spec)
}

func assertPipelineSpec(t *testing.T, spec plandsl.PipelineSpec) {
	t.Helper()
	if spec.Name != "release" {
		t.Fatalf("name = %q", spec.Name)
	}
	if len(spec.Stages) != 2 {
		t.Fatalf("stages = %d", len(spec.Stages))
	}
	assertLintStage(t, spec.Stages[0])
	assertTestStage(t, spec.Stages[1])
}

func assertLintStage(t *testing.T, lint plandsl.StageSpec) {
	t.Helper()
	if lint.Name != "lint" || lint.Image != "golangci:latest" || lint.TimeoutSeconds != 120 {
		t.Fatalf("unexpected lint stage: %+v", lint)
	}
}

func assertTestStage(t *testing.T, stage plandsl.StageSpec) {
	t.Helper()
	if len(stage.Needs) != 1 || stage.Needs[0] != "lint" {
		t.Fatalf("unexpected test needs: %+v", stage.Needs)
	}
	if len(stage.Script) != 1 || stage.Script[0] != "go test ./..." {
		t.Fatalf("unexpected test script: %+v", stage.Script)
	}
	if len(stage.Artifacts) != 1 || stage.Artifacts[0] != "coverage.out" {
		t.Fatalf("unexpected test artifacts: %+v", stage.Artifacts)
	}
}

func TestCompilePipelineDSLRejectsUnknownAction(t *testing.T) {
	t.Parallel()

	_, err := plandsl.Compile(context.Background(), "ci.plano", `
pipeline {
  name = "release"
}

stage lint {
  run {
    docker("golangci-lint run ./...")
  }
}
`)
	if err == nil {
		t.Fatalf("expected unknown action error")
	}
}
