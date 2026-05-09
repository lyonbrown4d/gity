package runneragent

import (
	"bytes"
	"context"
	"sync"
	"time"

	"github.com/samber/oops"
)

type cappedBuffer struct {
	mu            sync.Mutex
	buffer        bytes.Buffer
	limit         int
	truncated     bool
	truncatedSent bool
	ctx           context.Context
	started       time.Time
	traceStreamer ScriptTraceStreamer
}

func (b *cappedBuffer) Write(p []byte) (int, error) {
	b.mu.Lock()
	defer b.mu.Unlock()
	if b.limit <= 0 {
		return len(p), nil
	}
	remaining := b.limit - b.buffer.Len()
	if remaining <= 0 {
		err := b.markTruncated()
		return len(p), err
	}
	if len(p) > remaining {
		return b.writeTruncated(p, remaining)
	}
	return b.writeCaptured(p)
}

func (b *cappedBuffer) markTruncated() error {
	b.truncated = true
	if b.truncatedSent {
		return nil
	}
	b.truncatedSent = true
	if err := b.stream(nil); err != nil {
		return oops.In("runner_agent").Wrapf(err, "stream truncated job output")
	}
	return nil
}

func (b *cappedBuffer) writeTruncated(p []byte, remaining int) (int, error) {
	b.truncated = true
	b.truncatedSent = true
	captured := p[:remaining]
	if _, err := b.buffer.Write(captured); err != nil {
		return len(p), oops.In("runner_agent").With("remaining", remaining).Wrapf(err, "capture truncated job output")
	}
	if err := b.stream(captured); err != nil {
		return len(p), oops.In("runner_agent").Wrapf(err, "stream captured truncated job output")
	}
	return len(p), nil
}

func (b *cappedBuffer) writeCaptured(p []byte) (int, error) {
	n, err := b.buffer.Write(p)
	if streamErr := b.stream(p); streamErr != nil {
		if err != nil {
			return n, oops.In("runner_agent").Wrapf(oops.Join(err, streamErr), "capture and stream job output")
		}
		return n, oops.In("runner_agent").Wrapf(streamErr, "stream job output")
	}
	if err != nil {
		return n, oops.In("runner_agent").Wrapf(err, "capture job output")
	}
	return n, nil
}

func (b *cappedBuffer) String() string {
	b.mu.Lock()
	defer b.mu.Unlock()
	return b.buffer.String()
}

func (b *cappedBuffer) stream(chunk []byte) error {
	if b.traceStreamer == nil {
		return nil
	}
	if len(chunk) == 0 && !b.truncated {
		return nil
	}
	duration := int64(0)
	if !b.started.IsZero() {
		duration = time.Since(b.started).Milliseconds()
	}
	if err := b.traceStreamer(b.ctx, string(chunk), b.truncated, duration); err != nil {
		return oops.In("runner_agent").With("truncated", b.truncated, "duration_millis", duration).Wrapf(err, "stream job trace")
	}
	return nil
}
