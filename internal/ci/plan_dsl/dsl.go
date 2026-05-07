package plandsl

import (
	"context"
	"fmt"
	"regexp"
	"strings"

	"github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/plano/compiler"
	"github.com/arcgolabs/plano/schema"
)

const defaultTimeoutSeconds = 600

var safeShellArgPattern = regexp.MustCompile(`^[A-Za-z0-9_./:@%+=,-]+$`)

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
}

type CommandSpec struct {
	Name string   `json:"name"`
	Args []string `json:"args"`
}

func Compile(ctx context.Context, filename string, source string) (PipelineSpec, error) {
	c := compiler.New(compiler.Options{})
	if err := Register(c); err != nil {
		return PipelineSpec{}, err
	}
	result := c.CompileStringDetailed(ctx, filename, source)
	if result.Diagnostics.HasError() {
		return PipelineSpec{}, fmt.Errorf("compile ci plano: %s", result.Diagnostics.Error())
	}
	if result.HIR == nil {
		return PipelineSpec{}, fmt.Errorf("compile ci plano: missing HIR")
	}
	return Lower(result.HIR)
}

func Register(c *compiler.Compiler) error {
	if err := c.RegisterForms(forms()); err != nil {
		return fmt.Errorf("register ci plano forms: %w", err)
	}
	if err := c.RegisterActions(actions()); err != nil {
		return fmt.Errorf("register ci plano actions: %w", err)
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
		return PipelineSpec{}, fmt.Errorf("ci plano HIR is nil")
	}
	spec := PipelineSpec{}
	for idx := range hir.Forms.Len() {
		form, _ := hir.Forms.Get(idx)
		switch form.Kind {
		case "pipeline":
			name, err := requiredStringField(form, "name")
			if err != nil {
				return PipelineSpec{}, err
			}
			if spec.Name != "" {
				return PipelineSpec{}, fmt.Errorf("only one pipeline form is allowed")
			}
			spec.Name = name
		case "stage":
			stage, err := lowerStage(form)
			if err != nil {
				return PipelineSpec{}, err
			}
			spec.Stages = append(spec.Stages, stage)
		}
	}
	if spec.Name == "" {
		return PipelineSpec{}, fmt.Errorf("pipeline form is required")
	}
	if len(spec.Stages) == 0 {
		return PipelineSpec{}, fmt.Errorf("at least one stage form is required")
	}
	if err := validateStageGraph(spec.Stages); err != nil {
		return PipelineSpec{}, err
	}
	return spec, nil
}

func lowerStage(form compiler.HIRForm) (StageSpec, error) {
	if form.Symbol == nil {
		return StageSpec{}, fmt.Errorf("stage form requires symbol label")
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
	commands, err := lowerCommands(form)
	if err != nil {
		return StageSpec{}, err
	}
	script, err := commandsToScript(commands)
	if err != nil {
		return StageSpec{}, err
	}
	if len(script) == 0 {
		return StageSpec{}, fmt.Errorf("stage %q requires at least one shell or exec action", form.Symbol.Name)
	}
	return StageSpec{
		Name:           form.Symbol.Name,
		Needs:          needs,
		Image:          image,
		TimeoutSeconds: timeoutSeconds,
		Commands:       commands,
		Script:         script,
		Artifacts:      artifacts,
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
				return nil, fmt.Errorf("action %q expects string args, got %T", call.Name, arg.Value)
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
				return nil, fmt.Errorf("shell expects exactly one argument")
			}
			script = append(script, strings.TrimSpace(command.Args[0]))
		case "exec":
			if len(command.Args) == 0 {
				return nil, fmt.Errorf("exec expects at least one argument")
			}
			script = append(script, shellJoin(command.Args))
		default:
			return nil, fmt.Errorf("unsupported ci action: %s", command.Name)
		}
	}
	return script, nil
}

func shellJoin(args []string) string {
	quoted := make([]string, 0, len(args))
	for _, arg := range args {
		quoted = append(quoted, shellQuote(arg))
	}
	return strings.Join(quoted, " ")
}

func shellQuote(value string) string {
	if value != "" && safeShellArgPattern.MatchString(value) {
		return value
	}
	return "'" + strings.ReplaceAll(value, "'", "''") + "'"
}

func requiredStringField(form compiler.HIRForm, name string) (string, error) {
	value, err := stringField(form, name)
	if err != nil {
		return "", err
	}
	if value == "" {
		return "", fmt.Errorf("%s.%s is required", form.Kind, name)
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
		return "", fmt.Errorf("%s.%s must be string", form.Kind, name)
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
		return 0, fmt.Errorf("%s.%s must be int", form.Kind, name)
	}
}

func needsField(form compiler.HIRForm) ([]string, error) {
	field, ok := form.Field("needs")
	if !ok {
		return nil, nil
	}
	items, ok := field.Value.([]any)
	if !ok {
		return nil, fmt.Errorf("stage.needs must be list<ref<stage>>, got %T", field.Value)
	}
	needs := make([]string, 0, len(items))
	for _, item := range items {
		ref, ok := item.(schema.Ref)
		if !ok {
			return nil, fmt.Errorf("stage.needs expects ref<stage>, got %T", item)
		}
		if ref.Kind != "stage" {
			return nil, fmt.Errorf("stage.needs expects ref<stage>, got ref<%s>", ref.Kind)
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
		return nil, fmt.Errorf("%s.%s must be list<string>, got %T", form.Kind, name, field.Value)
	}
	values := make([]string, 0, len(items))
	for _, item := range items {
		text, ok := item.(string)
		if !ok {
			return nil, fmt.Errorf("%s.%s expects string, got %T", form.Kind, name, item)
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
				return fmt.Errorf("action %q expects string arguments, got %T", name, arg)
			}
		}
		return nil
	}
}

func validateStageGraph(stages []StageSpec) error {
	names := make(map[string]struct{}, len(stages))
	for _, stage := range stages {
		if _, exists := names[stage.Name]; exists {
			return fmt.Errorf("duplicate stage %q", stage.Name)
		}
		names[stage.Name] = struct{}{}
	}
	for _, stage := range stages {
		for _, need := range stage.Needs {
			if _, exists := names[need]; !exists {
				return fmt.Errorf("stage %q needs unknown stage %q", stage.Name, need)
			}
		}
	}
	visiting := map[string]bool{}
	visited := map[string]bool{}
	byName := make(map[string]StageSpec, len(stages))
	for _, stage := range stages {
		byName[stage.Name] = stage
	}
	var visit func(string) error
	visit = func(name string) error {
		if visited[name] {
			return nil
		}
		if visiting[name] {
			return fmt.Errorf("stage dependency cycle includes %q", name)
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
	for _, stage := range stages {
		if err := visit(stage.Name); err != nil {
			return err
		}
	}
	return nil
}
