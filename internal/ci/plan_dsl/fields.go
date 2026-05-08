package plandsl

import (
	"fmt"
	"strings"

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
	names := make(map[string]struct{}, len(stages))
	for i := range stages {
		stage := &stages[i]
		if _, exists := names[stage.Name]; exists {
			return oops.In("ci_plan_dsl").With("stage", stage.Name).New("duplicate stage")
		}
		names[stage.Name] = struct{}{}
	}
	for i := range stages {
		stage := &stages[i]
		for _, need := range stage.Needs {
			if _, exists := names[need]; !exists {
				return oops.In("ci_plan_dsl").With("stage", stage.Name, "need", need).New("stage needs unknown stage")
			}
		}
	}
	visiting := map[string]bool{}
	visited := map[string]bool{}
	byName := make(map[string]*StageSpec, len(stages))
	for i := range stages {
		stage := &stages[i]
		byName[stage.Name] = stage
	}
	var visit func(string) error
	visit = func(name string) error {
		if visited[name] {
			return nil
		}
		if visiting[name] {
			return oops.In("ci_plan_dsl").With("stage", name).New("stage dependency cycle")
		}
		visiting[name] = true
		for _, need := range byName[name].Needs {
			if err := visit(need); err != nil {
				return err
			}
		}
		visiting[name] = false
		visited[name] = true
		return nil
	}
	for i := range stages {
		stage := &stages[i]
		if err := visit(stage.Name); err != nil {
			return err
		}
	}
	return nil
}
