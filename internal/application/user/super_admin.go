package user

import (
	"context"
	"strings"

	"github.com/samber/oops"
)

func normalizeUpdateInput(id int64, input *UpdateInput) error {
	if input.Username != nil && strings.TrimSpace(*input.Username) == "" {
		return oops.In("user").With("user_id", id).New("username is required")
	}
	if input.DisplayName != nil && strings.TrimSpace(*input.DisplayName) == "" {
		displayName := ""
		if input.Username != nil {
			displayName = strings.TrimSpace(*input.Username)
		}
		input.DisplayName = &displayName
	}
	return nil
}

func (s *Service) ensureSuperAdminUpdateAllowed(ctx context.Context, id int64, input UpdateInput) error {
	if input.IsSuperAdmin == nil || *input.IsSuperAdmin {
		return nil
	}
	return s.ensureNotLastSuperAdmin(ctx, id, "demote user")
}

func (s *Service) ensureNotLastSuperAdmin(ctx context.Context, userID int64, operation string) error {
	user, err := s.repo.GetByID(ctx, userID)
	if err != nil {
		return oops.In("user").With("user_id", userID, "operation", operation).Wrapf(err, "load user before operation")
	}
	if user.IsSuperAdmin == 0 {
		return nil
	}
	count, err := s.countSuperAdmins(ctx)
	if err != nil {
		return err
	}
	if count <= 1 {
		return oops.In("user").With("user_id", userID, "operation", operation).New("cannot remove the last super admin")
	}
	return nil
}

func (s *Service) countSuperAdmins(ctx context.Context) (int, error) {
	users, err := s.repo.List(ctx)
	if err != nil {
		return 0, oops.In("user").Wrapf(err, "count super admins")
	}
	count := 0
	for _, user := range users.Values() {
		if user.IsSuperAdmin != 0 {
			count++
		}
	}
	return count, nil
}
