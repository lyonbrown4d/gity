// Package plandsl compiles Plano CI definitions into pipeline specs.
package plandsl

import (
	"context"
	"fmt"
	"strings"

	"github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/plano/compiler"
	"github.com/arcgolabs/plano/schema"
	"github.com/samber/oops"
)

const defaultTimeoutSeconds = 600

type PipelineSpec struct {
	Name   string      `json:"name"`
	Stages []StageSpec `json:"stages"`
}

type StageSpec struct {
	Name           string        `json:"name"`
	Needs          []string      `json:"needs"`
	Image          string        `json:"image,omitempty"`
	TimeoutSeconds int           `json:"timeout_seconds"`
	Commands       []CommandSpec `json:"commands"`
	Script         []string      `json:"script"`
	Artifacts      []string      `json:"artifacts,omitempty"`
	Tags           []string      `json:"tags,omitempty"`
}

type CommandSpec struct {
	Name string   `json:"name"`
	Args []string `json:"args"`
}

func Compile(ctx context.Context, filename, source string) (PipelineSpec, error) {
	c := compiler.New(compiler.Options{})
	if err := Register(c); err != nil {
		return PipelineSpec{}, err
	}
	result := c.CompileStringDetailed(ctx, filename, source)
	if result.Diagnostics.HasError() {
		return PipelineSpec{}, oops.In("ci_plan_dsl").With("filename", filename).Errorf("compile ci plano: %s", result.Diagnostics.Error())
	}
	if result.HIR == nil {
		return PipelineSpec{}, oops.In("ci_plan_dsl").With("filename", filename).New("compile ci plano: missing HIR")
	}
	return Lower(result.HIR)
}

func Register(c *compiler.Compiler) error {
	if err := c.RegisterForms(forms()); err != nil {
		return oops.In("ci_plan_dsl").Wrapf(err, "register ci plano forms")
	}
	if err := c.RegisterActions(actions()); err != nil {
		return oops.In("ci_plan_dsl").Wrapf(err, "register ci plano actions")
	}
	return nil
}

func forms() list.List[schema.FormSpec] {
	return schema.FormSpecs(
		schema.FormSpec{
			Name:      "pipeline",
			LabelKind: schema.LabelNone,
			BodyMode:  schema.BodyFieldOnly,
			Fields: schema.Fields(
				schema.FieldSpec{
					Name:     "name",
					Type:     schema.TypeString,
					Required: true,
				},
			),
		},
		schema.FormSpec{
			Name:         "stage",
			LabelKind:    schema.LabelSymbol,
			LabelRefKind: "stage",
			BodyMode:     schema.BodyScript,
			Declares:     "stage",
			Fields: schema.Fields(
				schema.FieldSpec{
					Name:       "needs",
					Type:       schema.ListType{Elem: schema.RefType{Kind: "stage"}},
					Default:    []any{},
					HasDefault: true,
				},
				schema.FieldSpec{
					Name:       "image",
					Type:       schema.TypeString,
					Default:    "",
					HasDefault: true,
				},
				schema.FieldSpec{
					Name:       "timeout_seconds",
					Type:       schema.TypeInt,
					Default:    int64(defaultTimeoutSeconds),
					HasDefault: true,
				},
				schema.FieldSpec{
					Name:       "artifacts",
					Type:       schema.ListType{Elem: schema.TypeString},
					Default:    []any{},
					HasDefault: true,
				},
				schema.FieldSpec{
					Name:       "tags",
					Type:       schema.ListType{Elem: schema.TypeString},
					Default:    []any{},
					HasDefault: true,
				},
			),
			NestedForms: schema.NestedForms("run"),
		},
		schema.FormSpec{
			Name:      "run",
			LabelKind: schema.LabelNone,
			BodyMode:  schema.BodyCallOnly,
		},
	)
}

func actions() list.List[compiler.ActionSpec] {
	return compiler.ActionSpecs(
		compiler.ActionSpec{
			Name:         "shell",
			MinArgs:      1,
			MaxArgs:      1,
			ArgTypes:     schema.Types(schema.TypeString),
			VariadicType: schema.TypeString,
			Validate:     validateStringArgs("shell"),
		},
		compiler.ActionSpec{
			Name:         "exec",
			MinArgs:      1,
			MaxArgs:      -1,
			ArgTypes:     schema.Types(schema.TypeString),
			VariadicType: schema.TypeString,
			Validate:     validateStringArgs("exec"),
		},
	)
}

func Lower(hir *compiler.HIR) (PipelineSpec, error) {
	if hir == nil {
		return PipelineSpec{}, oops.In("ci_plan_dsl").New("ci plano HIR is nil")
	}
	spec, err := lowerPipelineForms(hir)
	if err != nil {
		return PipelineSpec{}, err
	}
	if err := validatePipelineSpec(spec); err != nil {
		return PipelineSpec{}, err
	}
	return spec, nil
}

func lowerPipelineForms(hir *compiler.HIR) (PipelineSpec, error) {
	spec := PipelineSpec{}
	for idx := range hir.Forms.Len() {
		form, _ := hir.Forms.Get(idx)
		next, err := lowerPipelineForm(spec, form)
		if err != nil {
			return PipelineSpec{}, err
		}
		spec = next
	}
	return spec, nil
}

func lowerPipelineForm(spec PipelineSpec, form compiler.HIRForm) (PipelineSpec, error) {
	switch form.Kind {
	case "pipeline":
		name, err := requiredStringField(form, "name")
		if err != nil {
			return PipelineSpec{}, err
		}
		if spec.Name != "" {
			return PipelineSpec{}, oops.In("ci_plan_dsl").New("only one pipeline form is allowed")
		}
		spec.Name = name
	case "stage":
		stage, err := lowerStage(form)
		if err != nil {
			return PipelineSpec{}, err
		}
		spec.Stages = append(spec.Stages, stage)
	}
	return spec, nil
}

func validatePipelineSpec(spec PipelineSpec) error {
	if spec.Name == "" {
		return oops.In("ci_plan_dsl").New("pipeline form is required")
	}
	if len(spec.Stages) == 0 {
		return oops.In("ci_plan_dsl").New("at least one stage form is required")
	}
	return validateStageGraph(spec.Stages)
}

func lowerStage(form compiler.HIRForm) (StageSpec, error) {
	if form.Symbol == nil {
		return StageSpec{}, oops.In("ci_plan_dsl").With("form_kind", form.Kind).New("stage form requires symbol label")
	}
	needs, err := needsField(form)
	if err != nil {
		return StageSpec{}, err
	}
	image, err := stringField(form, "image")
	if err != nil {
		return StageSpec{}, err
	}
	timeoutSeconds, err := intField(form, "timeout_seconds")
	if err != nil {
		return StageSpec{}, err
	}
	artifacts, err := stringListField(form, "artifacts")
	if err != nil {
		return StageSpec{}, err
	}
	tags, err := stringListField(form, "tags")
	if err != nil {
		return StageSpec{}, err
	}
	commands, err := lowerCommands(form)
	if err != nil {
		return StageSpec{}, err
	}
	script, err := commandsToScript(commands)
	if err != nil {
		return StageSpec{}, err
	}
	if len(script) == 0 {
		return StageSpec{}, oops.In("ci_plan_dsl").With("stage", form.Symbol.Name).New("stage requires at least one shell or exec action")
	}
	return StageSpec{
		Name:           form.Symbol.Name,
		Needs:          needs,
		Image:          image,
		TimeoutSeconds: timeoutSeconds,
		Commands:       commands,
		Script:         script,
		Artifacts:      artifacts,
		Tags:           tags,
	}, nil
}

func lowerCommands(form compiler.HIRForm) ([]CommandSpec, error) {
	commands := make([]CommandSpec, 0, form.Calls.Len())
	direct, err := callsToCommands(form.Calls)
	if err != nil {
		return nil, err
	}
	commands = append(commands, direct...)
	for idx := range form.Forms.Len() {
		nested, _ := form.Forms.Get(idx)
		if nested.Kind != "run" {
			continue
		}
		items, err := callsToCommands(nested.Calls)
		if err != nil {
			return nil, err
		}
		commands = append(commands, items...)
	}
	return commands, nil
}

func callsToCommands(calls list.List[compiler.HIRCall]) ([]CommandSpec, error) {
	commands := make([]CommandSpec, 0, calls.Len())
	for idx := range calls.Len() {
		call, _ := calls.Get(idx)
		args := make([]string, 0, call.Args.Len())
		for argIdx := range call.Args.Len() {
			arg, _ := call.Args.Get(argIdx)
			text, ok := arg.Value.(string)
			if !ok {
				return nil, oops.In("ci_plan_dsl").With("action", call.Name, "arg_type", fmt.Sprintf("%T", arg.Value)).New("action expects string args")
			}
			args = append(args, text)
		}
		commands = append(commands, CommandSpec{Name: call.Name, Args: args})
	}
	return commands, nil
}

func commandsToScript(commands []CommandSpec) ([]string, error) {
	script := make([]string, 0, len(commands))
	for _, command := range commands {
		switch command.Name {
		case "shell":
			if len(command.Args) != 1 {
				return nil, oops.In("ci_plan_dsl").With("action", command.Name, "arg_count", len(command.Args)).New("shell expects exactly one argument")
			}
			script = append(script, strings.TrimSpace(command.Args[0]))
		case "exec":
			if len(command.Args) == 0 {
				return nil, oops.In("ci_plan_dsl").With("action", command.Name).New("exec expects at least one argument")
			}
			script = append(script, shellJoin(command.Args))
		default:
			return nil, oops.In("ci_plan_dsl").With("action", command.Name).New("unsupported ci action")
		}
	}
	return script, nil
}
