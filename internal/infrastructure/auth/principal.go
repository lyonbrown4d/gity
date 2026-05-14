package auth

type Principal struct {
	UserID       int64
	Username     string
	IsSuperAdmin bool
}
