package auth

type Principal struct {
	UserID       int64
	Username     string
	IsSuperAdmin bool
	TokenKind    string
	ProjectID    int64
	Scopes       string
}
