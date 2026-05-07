package plandsl

import (
	"context"
	"testing"
)

func TestCompilePipelineDSL(t *testing.T) {
	t.Parallel()

	spec, err := Compile(context.Background(), "ci.plano", `
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
	if spec.Name != "release" {
		t.Fatalf("name = %q", spec.Name)
	}
	if len(spec.Stages) != 2 {
		t.Fatalf("stages = %d", len(spec.Stages))
	}
	lint := spec.Stages[0]
	if lint.Name != "lint" || lint.Image != "golangci:latest" || lint.TimeoutSeconds != 120 {
		t.Fatalf("unexpected lint stage: %+v", lint)
	}
	test := spec.Stages[1]
	if len(test.Needs) != 1 || test.Needs[0] != "lint" {
		t.Fatalf("unexpected test needs: %+v", test.Needs)
	}
	if len(test.Script) != 1 || test.Script[0] != "go test ./..." {
		t.Fatalf("unexpected test script: %+v", test.Script)
	}
	if len(test.Artifacts) != 1 || test.Artifacts[0] != "coverage.out" {
		t.Fatalf("unexpected test artifacts: %+v", test.Artifacts)
	}
}

func TestCompilePipelineDSLRejectsUnknownAction(t *testing.T) {
	t.Parallel()

	_, err := Compile(context.Background(), "ci.plano", `
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
