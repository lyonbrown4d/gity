package user

import (
	"context"

	collectionlist "github.com/arcgolabs/collectionx/list"
	userservice "github.com/lyonbrown4d/gity/internal/application/user"
	identity "github.com/lyonbrown4d/gity/internal/domain/identity"
)

func (e *Endpoint) listUsers(ctx context.Context, in *usersInput) (*userOutput, error) {
	items, err := e.service.List(ctx)
	if err != nil {
		return nil, err
	}
	idFilter := parseIDFilter(in.IDs)
	views := collectionlist.FilterMapList(items, func(_ int, item identity.User) (userView, bool) {
		if idFilter.Len() > 0 && !idFilter.Contains(item.ID) {
			return userView{}, false
		}
		return toUserView(item), true
	}).Values()
	return &userOutput{Body: views}, nil
}

func (e *Endpoint) getCurrentUser(ctx context.Context, in *currentUserInput) (*userOutput, error) {
	user, err := currentUser(ctx, e.service, e.authRuntime, in.Authorization)
	if err != nil {
		return nil, err
	}
	return &userOutput{Body: toUserView(user)}, nil
}

func (e *Endpoint) getUser(ctx context.Context, in *userByIDInput) (*userOutput, error) {
	item, err := e.service.GetByID(ctx, in.ID)
	if err != nil {
		return nil, err
	}
	return &userOutput{Body: toUserView(item)}, nil
}

func (e *Endpoint) createUser(ctx context.Context, in *createUserInput) (*userOutput, error) {
	item, err := e.service.Create(ctx, userservice.CreateInput{
		Username:    in.Body.Username,
		DisplayName: in.Body.DisplayName,
		Email:       in.Body.Email,
	})
	if err != nil {
		return nil, err
	}
	return &userOutput{Body: toUserView(item)}, nil
}

func (e *Endpoint) updateCurrentUser(ctx context.Context, in *updateCurrentUserInput) (*userOutput, error) {
	user, err := currentUser(ctx, e.service, e.authRuntime, in.Authorization)
	if err != nil {
		return nil, err
	}
	return e.updateUserByID(ctx, user.ID, in.Body)
}

func (e *Endpoint) updateUser(ctx context.Context, in *updateUserInput) (*userOutput, error) {
	return e.updateUserByID(ctx, in.ID, in.Body)
}

func (e *Endpoint) deleteUser(ctx context.Context, in *userByIDInput) (*userOutput, error) {
	item, err := e.service.GetByID(ctx, in.ID)
	if err != nil {
		return nil, err
	}
	if err := e.service.Delete(ctx, in.ID); err != nil {
		return nil, err
	}
	return &userOutput{Body: toUserView(item)}, nil
}

func (e *Endpoint) listTokens(ctx context.Context, in *userTokenInput) (*userOutput, error) {
	items, err := e.service.ListTokens(ctx, in.ID)
	if err != nil {
		return nil, err
	}
	return &userOutput{Body: items}, nil
}

func (e *Endpoint) createToken(ctx context.Context, in *createUserTokenInput) (*userOutput, error) {
	item, err := e.service.CreateToken(ctx, in.ID, userservice.CreateTokenInput{
		Name: in.Body.Name,
	})
	if err != nil {
		return nil, err
	}
	return &userOutput{Body: item}, nil
}

func (e *Endpoint) updateUserByID(ctx context.Context, id int64, body updateUserBody) (*userOutput, error) {
	item, err := e.service.Update(ctx, id, userservice.UpdateInput{
		Username:    body.Username,
		DisplayName: body.DisplayName,
		Email:       body.Email,
		Status:      body.Status,
	})
	if err != nil {
		return nil, err
	}
	return &userOutput{Body: toUserView(item)}, nil
}
