package plandsl

import (
	"fmt"
	"strings"

	"github.com/arcgolabs/collectionx/graph"
	"github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/plano/compiler"
	"github.com/arcgolabs/plano/schema"
	"github.com/samber/oops"
)

func requiredStringField(form compiler.HIRForm, name string) (string, error) {
	value, err := stringField(form, name)
	if err != nil {
		return "", err
	}
	if value == "" {
		return "", oops.In("ci_plan_dsl").With("form_kind", form.Kind, "field", name).New("field is required")
	}
	return value, nil
}

func stringField(form compiler.HIRForm, name string) (string, error) {
	field, ok := form.Field(name)
	if !ok {
		return "", nil
	}
	value, ok := field.Value.(string)
	if !ok {
		return "", oops.In("ci_plan_dsl").With("form_kind", form.Kind, "field", name).New("field must be string")
	}
	return strings.TrimSpace(value), nil
}

func intField(form compiler.HIRForm, name string) (int, error) {
	field, ok := form.Field(name)
	if !ok {
		return 0, nil
	}
	switch value := field.Value.(type) {
	case int:
		return value, nil
	case int64:
		return int(value), nil
	default:
		return 0, oops.In("ci_plan_dsl").With("form_kind", form.Kind, "field", name).New("field must be int")
	}
}

func needsField(form compiler.HIRForm) ([]string, error) {
	field, ok := form.Field("needs")
	if !ok {
		return nil, nil
	}
	items, ok := field.Value.([]any)
	if !ok {
		return nil, oops.In("ci_plan_dsl").With("field", "stage.needs", "value_type", fmt.Sprintf("%T", field.Value)).New("stage.needs must be list<ref<stage>>")
	}
	needs := make([]string, 0, len(items))
	for _, item := range items {
		ref, ok := item.(schema.Ref)
		if !ok {
			return nil, oops.In("ci_plan_dsl").With("field", "stage.needs", "value_type", fmt.Sprintf("%T", item)).New("stage.needs expects ref<stage>")
		}
		if ref.Kind != "stage" {
			return nil, oops.In("ci_plan_dsl").With("field", "stage.needs", "ref_kind", ref.Kind).New("stage.needs expects ref<stage>")
		}
		needs = append(needs, ref.Name)
	}
	return needs, nil
}

func stringListField(form compiler.HIRForm, name string) ([]string, error) {
	field, ok := form.Field(name)
	if !ok {
		return nil, nil
	}
	items, ok := field.Value.([]any)
	if !ok {
		return nil, oops.In("ci_plan_dsl").With("form_kind", form.Kind, "field", name, "value_type", fmt.Sprintf("%T", field.Value)).New("field must be list<string>")
	}
	values := make([]string, 0, len(items))
	for _, item := range items {
		text, ok := item.(string)
		if !ok {
			return nil, oops.In("ci_plan_dsl").With("form_kind", form.Kind, "field", name, "value_type", fmt.Sprintf("%T", item)).New("field expects string")
		}
		text = strings.TrimSpace(text)
		if text != "" {
			values = append(values, text)
		}
	}
	return values, nil
}

func validateStringArgs(name string) func(args list.List[any]) error {
	return func(args list.List[any]) error {
		for _, arg := range args.Values() {
			if _, ok := arg.(string); !ok {
				return oops.In("ci_plan_dsl").With("action", name, "arg_type", fmt.Sprintf("%T", arg)).New("action expects string arguments")
			}
		}
		return nil
	}
}

func validateStageGraph(stages []StageSpec) error {
	stageGraph, err := buildStageGraph(stages)
	if err != nil {
		return err
	}
	if _, err := stageGraph.TopologicalSort(); err != nil {
		return oops.In("ci_plan_dsl").Wrapf(err, "stage dependency cycle")
	}
	return nil
}

func buildStageGraph(stages []StageSpec) (*graph.Graph[string, StageSpec], error) {
	stageGraph := graph.NewDirectedGraph[string, StageSpec]()
	if err := addStageGraphNodes(stageGraph, stages); err != nil {
		return nil, err
	}
	if err := addStageGraphEdges(stageGraph, stages); err != nil {
		return nil, err
	}
	return stageGraph, nil
}

func addStageGraphNodes(stageGraph *graph.Graph[string, StageSpec], stages []StageSpec) error {
	for i := range stages {
		stage := &stages[i]
		if !stageGraph.AddNode(stage.Name, *stage) {
			return oops.In("ci_plan_dsl").With("stage", stage.Name).New("duplicate stage")
		}
	}
	return nil
}

func addStageGraphEdges(stageGraph *graph.Graph[string, StageSpec], stages []StageSpec) error {
	for i := range stages {
		stage := &stages[i]
		for _, need := range stage.Needs {
			if !stageGraph.HasNode(need) {
				return oops.In("ci_plan_dsl").With("stage", stage.Name, "need", need).New("stage needs unknown stage")
			}
			if err := stageGraph.AddEdge(need, stage.Name); err != nil {
				return oops.In("ci_plan_dsl").With("stage", stage.Name, "need", need).Wrapf(err, "add stage dependency")
			}
		}
	}
	return nil
}
