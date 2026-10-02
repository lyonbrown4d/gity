package todo

import (
	"context"
	"testing"

	"github.com/arcgolabs/eventx"
	projectdomain "github.com/lyonbrown4d/gity/internal/domain/project"
)

func TestSubscriberSubscribeTodoEventClosesGenericSubscription(t *testing.T) {
	t.Parallel()

	bus := eventx.New(eventx.WithParallelDispatch(false))
	subscriber := &Subscriber{}
	handled := 0
	if err := subscriber.subscribeTodoEvent[projectdomain.ProjectCreated](bus, func(context.Context, projectdomain.ProjectCreated) error {
		handled++
		return nil
	}); err != nil {
		t.Fatalf("subscribe project created event: %v", err)
	}

	event := projectdomain.ProjectCreated{ProjectID: 1}
	if err := bus.Publish(t.Context(), event); err != nil {
		t.Fatalf("publish subscribed event: %v", err)
	}
	if handled != 1 {
		t.Fatalf("handled events = %d, want 1", handled)
	}

	subscriber.Close()
	if err := bus.Publish(t.Context(), event); err != nil {
		t.Fatalf("publish unsubscribed event: %v", err)
	}
	if handled != 1 {
		t.Fatalf("handled events after close = %d, want 1", handled)
	}
}
