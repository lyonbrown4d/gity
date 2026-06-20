import { useEffect, useMemo, useState } from "react";
import { Copy, KeyRound, Plus, RefreshCw, ShieldCheck, Trash2 } from "lucide-react";
import { useCustom, useCustomMutation } from "@refinedev/core";
import { ConfirmAction } from "@/components/common/confirm-action";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import type {
  CreatedRepositoryProjectTokenView,
  RepositoryDeployKeyView,
  RepositoryProjectTokenView,
} from "@/pages/types";
import { extractErrorMessage } from "./issues-utils";
import type { RepositoryPermissions } from "./repository-permissions";
import {
  isRecord,
  normalizeBoolean,
  normalizeOptionalString,
  normalizeString,
  normalizeStringArray,
  resolveBody,
  type RawRecord,
} from "./repository-normalizers";

interface RepositoryCredentialsPanelProps {
  repoId: string;
  permissions: RepositoryPermissions;
  t: (text: string) => string;
  onError: (message: string | null) => void;
}

const accessTokenScopes = ["read_repository", "write_repository", "read_package", "write_package", "read_api", "write_api"];
const deployTokenScopes = ["read_repository", "write_repository", "read_package", "write_package"];

export const RepositoryCredentialsPanel = ({ repoId, permissions, t, onError }: RepositoryCredentialsPanelProps): JSX.Element => {
  const accessTokensQuery = useCustom<RawRecord[]>({
    url: `/projects/${repoId}/access-tokens`,
    method: "get",
    queryOptions: {
      enabled: Boolean(repoId),
      refetchOnWindowFocus: false,
    },
  });
  const deployTokensQuery = useCustom<RawRecord[]>({
    url: `/projects/${repoId}/deploy-tokens`,
    method: "get",
    queryOptions: {
      enabled: Boolean(repoId),
      refetchOnWindowFocus: false,
    },
  });
  const deployKeysQuery = useCustom<RawRecord[]>({
    url: `/projects/${repoId}/deploy-keys`,
    method: "get",
    queryOptions: {
      enabled: Boolean(repoId),
      refetchOnWindowFocus: false,
    },
  });
  const { mutateAsync: createToken, mutation: { isPending: isCreatingToken } } = useCustomMutation<RawRecord>();
  const { mutateAsync: revokeToken, mutation: { isPending: isRevokingToken } } = useCustomMutation<RawRecord>();
  const { mutateAsync: createKey, mutation: { isPending: isCreatingKey } } = useCustomMutation<RawRecord>();
  const { mutateAsync: deleteKey, mutation: { isPending: isDeletingKey } } = useCustomMutation<RawRecord>();
  const [accessTokenName, setAccessTokenName] = useState("");
  const [accessTokenScopesValue, setAccessTokenScopesValue] = useState<string[]>(["read_repository"]);
  const [accessTokenExpiresAt, setAccessTokenExpiresAt] = useState("");
  const [deployTokenName, setDeployTokenName] = useState("");
  const [deployTokenUsername, setDeployTokenUsername] = useState("");
  const [deployTokenScopesValue, setDeployTokenScopesValue] = useState<string[]>(["read_repository"]);
  const [deployTokenExpiresAt, setDeployTokenExpiresAt] = useState("");
  const [deployKeyTitle, setDeployKeyTitle] = useState("");
  const [deployKeyPublicKey, setDeployKeyPublicKey] = useState("");
  const [deployKeyCanPush, setDeployKeyCanPush] = useState(false);
  const [createdToken, setCreatedToken] = useState<CreatedRepositoryProjectTokenView | null>(null);
  const canAdminCredentials = permissions.repositoryAdmin;
  const isBusy = isCreatingToken || isRevokingToken || isCreatingKey || isDeletingKey;
  const accessTokens = useMemo(
    () => resolveCredentialRecords(accessTokensQuery.result.data).map(normalizeProjectToken),
    [accessTokensQuery.result.data],
  );
  const deployTokens = useMemo(
    () => resolveCredentialRecords(deployTokensQuery.result.data).map(normalizeProjectToken),
    [deployTokensQuery.result.data],
  );
  const deployKeys = useMemo(
    () => resolveCredentialRecords(deployKeysQuery.result.data).map(normalizeDeployKey),
    [deployKeysQuery.result.data],
  );

  const reload = async () => {
    const results = await Promise.all([accessTokensQuery.query.refetch(), deployTokensQuery.query.refetch(), deployKeysQuery.query.refetch()]);
    const error = results.find((result) => result.error)?.error;
    onError(error ? extractErrorMessage(error) : null);
  };

  const submitCreateAccessToken = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!accessTokenName.trim()) {
      onError(t("Project access token name is required."));
      return;
    }
    await submitCreateToken({
      url: `/projects/${repoId}/access-tokens`,
      values: {
        name: accessTokenName,
        scopes: accessTokenScopesValue,
        expires_at: dateToRFC3339(accessTokenExpiresAt),
      },
      onDone: () => {
        setAccessTokenName("");
        setAccessTokenScopesValue(["read_repository"]);
        setAccessTokenExpiresAt("");
      },
    });
  };

  const submitCreateDeployToken = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!deployTokenName.trim()) {
      onError(t("Deploy token name is required."));
      return;
    }
    await submitCreateToken({
      url: `/projects/${repoId}/deploy-tokens`,
      values: {
        name: deployTokenName,
        username: deployTokenUsername,
        scopes: deployTokenScopesValue,
        expires_at: dateToRFC3339(deployTokenExpiresAt),
      },
      onDone: () => {
        setDeployTokenName("");
        setDeployTokenUsername("");
        setDeployTokenScopesValue(["read_repository"]);
        setDeployTokenExpiresAt("");
      },
    });
  };

  const submitCreateToken = async ({
    url,
    values,
    onDone,
  }: {
    url: string;
    values: Record<string, unknown>;
    onDone: () => void;
  }) => {
    onError(null);
    try {
      const result = await createToken({
        url,
        method: "post",
        values,
      });
      const normalized = normalizeCreatedToken(result.data);
      setCreatedToken(normalized);
      onDone();
      await reload();
    } catch (error) {
      onError(extractErrorMessage(error));
    }
  };

  const submitRevokeToken = async (token: RepositoryProjectTokenView, kind: "access" | "deploy") => {
    onError(null);
    try {
      await revokeToken({
        url: `/projects/${repoId}/${kind === "access" ? "access-tokens" : "deploy-tokens"}/${token.id}`,
        method: "delete",
        values: {},
      });
      await reload();
    } catch (error) {
      onError(extractErrorMessage(error));
    }
  };

  const submitCreateDeployKey = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!deployKeyTitle.trim() || !deployKeyPublicKey.trim()) {
      onError(t("Deploy key title and public key are required."));
      return;
    }
    onError(null);
    try {
      await createKey({
        url: `/projects/${repoId}/deploy-keys`,
        method: "post",
        values: {
          title: deployKeyTitle,
          public_key: deployKeyPublicKey,
          can_push: deployKeyCanPush,
        },
      });
      setDeployKeyTitle("");
      setDeployKeyPublicKey("");
      setDeployKeyCanPush(false);
      await reload();
    } catch (error) {
      onError(extractErrorMessage(error));
    }
  };

  const submitDeleteDeployKey = async (key: RepositoryDeployKeyView) => {
    onError(null);
    try {
      await deleteKey({
        url: `/projects/${repoId}/deploy-keys/${key.id}`,
        method: "delete",
        values: {},
      });
      await reload();
    } catch (error) {
      onError(extractErrorMessage(error));
    }
  };

  const copyCreatedToken = async () => {
    if (!createdToken) {
      return;
    }
    try {
      await navigator.clipboard.writeText(createdToken.token);
      onError(null);
    } catch {
      onError(t("Failed to copy token."));
    }
  };

  useEffect(() => {
    const error = accessTokensQuery.query.error ?? deployTokensQuery.query.error ?? deployKeysQuery.query.error;
    if (error) {
      onError(extractErrorMessage(error));
    }
  }, [accessTokensQuery.query.error, deployTokensQuery.query.error, deployKeysQuery.query.error, onError]);

  return (
    <Card className="card-enter">
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <ShieldCheck className="size-4" />
          {t("Project credentials")}
        </CardTitle>
        <CardDescription>{t("Manage project-scoped tokens and deploy keys for Git, packages and automation.")}</CardDescription>
      </CardHeader>
      <CardContent className="space-y-5">
        {!canAdminCredentials ? (
          <Alert>
            <AlertDescription>{t("Your current project role can inspect settings, but cannot change project credentials.")}</AlertDescription>
          </Alert>
        ) : null}

        {createdToken ? (
          <Alert>
            <KeyRound className="size-4" />
            <AlertDescription className="space-y-2">
              <p className="font-medium">{t("Token created. Copy it now because it will not be shown again.")}</p>
              <div className="flex flex-wrap items-center gap-2">
                <code className="rounded bg-muted px-2 py-1 text-xs">{createdToken.token}</code>
                <Button type="button" size="sm" variant="outline" onClick={() => void copyCreatedToken()}>
                  <Copy className="size-4" />
                  {t("Copy")}
                </Button>
                <Button type="button" size="sm" variant="ghost" onClick={() => setCreatedToken(null)}>
                  {t("Dismiss")}
                </Button>
              </div>
            </AlertDescription>
          </Alert>
        ) : null}

        <section className="space-y-3 rounded-md border p-3">
          <CredentialSectionHeader title={t("Project access tokens")} description={t("Use project tokens for API, repository and package automation scoped to this project.")} onReload={() => void reload()} t={t} />
          <TokenForm
            idPrefix="project-access-token"
            name={accessTokenName}
            expiresAt={accessTokenExpiresAt}
            scopes={accessTokenScopesValue}
            scopeOptions={accessTokenScopes}
            disabled={!canAdminCredentials || isBusy}
            namePlaceholder={t("Release bot")}
            t={t}
            onSubmit={submitCreateAccessToken}
            onNameChange={setAccessTokenName}
            onExpiresAtChange={setAccessTokenExpiresAt}
            onScopesChange={setAccessTokenScopesValue}
          />
          <TokenList
            tokens={accessTokens}
            emptyText={t("No project access tokens yet.")}
            isLoading={accessTokensQuery.query.isFetching && !accessTokensQuery.query.data}
            disabled={!canAdminCredentials || isBusy}
            t={t}
            onRevoke={(token) => void submitRevokeToken(token, "access")}
          />
        </section>

        <section className="space-y-3 rounded-md border p-3">
          <CredentialSectionHeader title={t("Deploy tokens")} description={t("Use deploy tokens for Git and package pull/push flows without granting full user access.")} onReload={() => void reload()} t={t} />
          <TokenForm
            idPrefix="deploy-token"
            name={deployTokenName}
            username={deployTokenUsername}
            expiresAt={deployTokenExpiresAt}
            scopes={deployTokenScopesValue}
            scopeOptions={deployTokenScopes}
            disabled={!canAdminCredentials || isBusy}
            namePlaceholder={t("Production deploy")}
            usernamePlaceholder={t("deploy-production")}
            t={t}
            onSubmit={submitCreateDeployToken}
            onNameChange={setDeployTokenName}
            onUsernameChange={setDeployTokenUsername}
            onExpiresAtChange={setDeployTokenExpiresAt}
            onScopesChange={setDeployTokenScopesValue}
          />
          <TokenList
            tokens={deployTokens}
            emptyText={t("No deploy tokens yet.")}
            isLoading={deployTokensQuery.query.isFetching && !deployTokensQuery.query.data}
            disabled={!canAdminCredentials || isBusy}
            t={t}
            onRevoke={(token) => void submitRevokeToken(token, "deploy")}
          />
        </section>

        <section className="space-y-3 rounded-md border p-3">
          <CredentialSectionHeader title={t("Deploy keys")} description={t("Store SSH public keys for future SSH transport integration and repository deploy policy.")} onReload={() => void reload()} t={t} />
          <form className="grid gap-3 rounded-md border bg-muted/10 p-3 lg:grid-cols-[220px_1fr_auto]" onSubmit={submitCreateDeployKey}>
            <div className="space-y-1">
              <Label className="text-xs text-muted-foreground" htmlFor="deploy-key-title">
                {t("Title")}
              </Label>
              <Input id="deploy-key-title" value={deployKeyTitle} onChange={(event) => setDeployKeyTitle(event.target.value)} placeholder={t("Build server")} disabled={!canAdminCredentials || isBusy} />
            </div>
            <div className="space-y-1">
              <Label className="text-xs text-muted-foreground" htmlFor="deploy-key-public-key">
                {t("Public key")}
              </Label>
              <Textarea id="deploy-key-public-key" value={deployKeyPublicKey} onChange={(event) => setDeployKeyPublicKey(event.target.value)} placeholder="ssh-ed25519 AAAA..." disabled={!canAdminCredentials || isBusy} />
            </div>
            <div className="flex flex-wrap items-end gap-2">
              <Button type="button" variant={deployKeyCanPush ? "secondary" : "outline"} disabled={!canAdminCredentials || isBusy} onClick={() => setDeployKeyCanPush((current) => !current)}>
                {t("Can push")}
              </Button>
              <Button type="submit" disabled={!canAdminCredentials || isBusy || !deployKeyTitle.trim() || !deployKeyPublicKey.trim()}>
                <Plus className="size-4" />
                {isCreatingKey ? t("Adding...") : t("Add key")}
              </Button>
            </div>
          </form>
          <DeployKeyList
            keys={deployKeys}
            emptyText={t("No deploy keys yet.")}
            isLoading={deployKeysQuery.query.isFetching && !deployKeysQuery.query.data}
            disabled={!canAdminCredentials || isBusy}
            t={t}
            onDelete={(key) => void submitDeleteDeployKey(key)}
          />
        </section>
      </CardContent>
    </Card>
  );
};

const CredentialSectionHeader = ({
  title,
  description,
  onReload,
  t,
}: {
  title: string;
  description: string;
  onReload: () => void;
  t: (text: string) => string;
}) => (
  <div className="flex flex-wrap items-start justify-between gap-2">
    <div>
      <p className="font-medium">{title}</p>
      <p className="text-xs text-muted-foreground">{description}</p>
    </div>
    <Button type="button" size="sm" variant="ghost" onClick={onReload}>
      <RefreshCw className="size-4" />
      {t("Reload")}
    </Button>
  </div>
);

const TokenForm = ({
  idPrefix,
  name,
  username,
  expiresAt,
  scopes,
  scopeOptions,
  disabled,
  namePlaceholder,
  usernamePlaceholder,
  t,
  onSubmit,
  onNameChange,
  onUsernameChange,
  onExpiresAtChange,
  onScopesChange,
}: {
  idPrefix: string;
  name: string;
  username?: string;
  expiresAt: string;
  scopes: string[];
  scopeOptions: string[];
  disabled: boolean;
  namePlaceholder: string;
  usernamePlaceholder?: string;
  t: (text: string) => string;
  onSubmit: (event: React.FormEvent<HTMLFormElement>) => void;
  onNameChange: (value: string) => void;
  onUsernameChange?: (value: string) => void;
  onExpiresAtChange: (value: string) => void;
  onScopesChange: (value: string[]) => void;
}) => (
  <form className="grid gap-3 rounded-md border bg-muted/10 p-3 lg:grid-cols-[220px_220px_1fr_auto]" onSubmit={onSubmit}>
    <div className="space-y-1">
      <Label className="text-xs text-muted-foreground" htmlFor={`${idPrefix}-name`}>
        {t("Name")}
      </Label>
      <Input id={`${idPrefix}-name`} value={name} onChange={(event) => onNameChange(event.target.value)} placeholder={namePlaceholder} disabled={disabled} />
    </div>
    {onUsernameChange ? (
      <div className="space-y-1">
        <Label className="text-xs text-muted-foreground" htmlFor={`${idPrefix}-username`}>
          {t("Username")}
        </Label>
        <Input id={`${idPrefix}-username`} value={username ?? ""} onChange={(event) => onUsernameChange(event.target.value)} placeholder={usernamePlaceholder} disabled={disabled} />
      </div>
    ) : null}
    <div className="space-y-1">
      <Label className="text-xs text-muted-foreground" htmlFor={`${idPrefix}-expires`}>
        {t("Expires at")}
      </Label>
      <Input id={`${idPrefix}-expires`} value={expiresAt} onChange={(event) => onExpiresAtChange(event.target.value)} type="date" disabled={disabled} />
    </div>
    <div className="flex flex-wrap items-end gap-2">
      <Button type="submit" disabled={disabled || !name.trim() || scopes.length === 0}>
        <Plus className="size-4" />
        {t("Create token")}
      </Button>
    </div>
    <div className="lg:col-span-4">
      <ScopePicker values={scopes} options={scopeOptions} disabled={disabled} t={t} onChange={onScopesChange} />
    </div>
  </form>
);

const ScopePicker = ({
  values,
  options,
  disabled,
  t,
  onChange,
}: {
  values: string[];
  options: string[];
  disabled: boolean;
  t: (text: string) => string;
  onChange: (values: string[]) => void;
}) => {
  const toggle = (scope: string) => {
    if (values.includes(scope)) {
      onChange(values.filter((item) => item !== scope));
      return;
    }
    onChange([...values, scope]);
  };
  return (
    <div className="flex flex-wrap gap-2">
      {options.map((scope) => (
        <Button key={scope} type="button" size="sm" variant={values.includes(scope) ? "secondary" : "outline"} disabled={disabled} onClick={() => toggle(scope)}>
          {t(scope)}
        </Button>
      ))}
    </div>
  );
};

const TokenList = ({
  tokens,
  emptyText,
  isLoading,
  disabled,
  t,
  onRevoke,
}: {
  tokens: RepositoryProjectTokenView[];
  emptyText: string;
  isLoading: boolean;
  disabled: boolean;
  t: (text: string) => string;
  onRevoke: (token: RepositoryProjectTokenView) => void;
}) => (
  <div className="space-y-2 rounded-md border p-2">
    {isLoading ? <p className="px-2 py-2 text-sm text-muted-foreground">{t("Loading credentials...")}</p> : null}
    {tokens.length === 0 ? <p className="px-2 py-2 text-sm text-muted-foreground">{emptyText}</p> : null}
    {tokens.map((token) => (
      <div key={token.id} className="flex flex-wrap items-center justify-between gap-3 rounded-md border p-3">
        <div className="min-w-0">
          <div className="flex flex-wrap items-center gap-2">
            <p className="font-medium">{token.name}</p>
            <Badge variant={token.active ? "secondary" : "outline"}>{token.active ? t("active") : t("revoked")}</Badge>
          </div>
          <p className="text-xs text-muted-foreground">@{token.username || token.id}</p>
          <div className="mt-2 flex flex-wrap gap-1">
            {token.scopes.map((scope) => (
              <Badge key={scope} variant="outline">{t(scope)}</Badge>
            ))}
          </div>
          <p className="mt-2 text-xs text-muted-foreground">
            {t("Expires at")}: {token.expires_at || t("Never")} · {t("Last used")}: {token.last_used_at || t("Never")}
          </p>
        </div>
        <ConfirmAction
          title={t("Revoke token?")}
          description={t("Existing automation using this token will stop working immediately.")}
          confirmLabel={t("Revoke")}
          cancelLabel={t("Cancel")}
          onConfirm={() => onRevoke(token)}
        >
          <Button type="button" size="sm" variant="outline" disabled={disabled || !token.active}>
            <Trash2 className="size-4" />
            {t("Revoke")}
          </Button>
        </ConfirmAction>
      </div>
    ))}
  </div>
);

const DeployKeyList = ({
  keys,
  emptyText,
  isLoading,
  disabled,
  t,
  onDelete,
}: {
  keys: RepositoryDeployKeyView[];
  emptyText: string;
  isLoading: boolean;
  disabled: boolean;
  t: (text: string) => string;
  onDelete: (key: RepositoryDeployKeyView) => void;
}) => (
  <div className="space-y-2 rounded-md border p-2">
    {isLoading ? <p className="px-2 py-2 text-sm text-muted-foreground">{t("Loading deploy keys...")}</p> : null}
    {keys.length === 0 ? <p className="px-2 py-2 text-sm text-muted-foreground">{emptyText}</p> : null}
    {keys.map((key) => (
      <div key={key.id} className="flex flex-wrap items-center justify-between gap-3 rounded-md border p-3">
        <div className="min-w-0">
          <div className="flex flex-wrap items-center gap-2">
            <p className="font-medium">{key.title}</p>
            {key.can_push ? <Badge variant="secondary">{t("can push")}</Badge> : <Badge variant="outline">{t("read only")}</Badge>}
          </div>
          <p className="text-xs text-muted-foreground">{key.fingerprint}</p>
          <p className="mt-2 max-w-5xl break-all font-mono text-xs text-muted-foreground">{key.public_key}</p>
        </div>
        <ConfirmAction
          title={t("Delete deploy key?")}
          description={t("Deployments using this key will no longer be trusted by this project.")}
          confirmLabel={t("Delete")}
          cancelLabel={t("Cancel")}
          onConfirm={() => onDelete(key)}
        >
          <Button type="button" size="sm" variant="outline" disabled={disabled}>
            <Trash2 className="size-4" />
            {t("Delete")}
          </Button>
        </ConfirmAction>
      </div>
    ))}
  </div>
);

const resolveCredentialRecords = (payload: unknown): RawRecord[] => {
  const body = resolveBody(payload);
  if (Array.isArray(body)) {
    return body.filter(isRecord);
  }
  return [];
};

const normalizeProjectToken = (raw: RawRecord): RepositoryProjectTokenView => ({
  id: normalizeString(raw.id ?? raw.ID),
  project_id: normalizeString(raw.project_id ?? raw.ProjectID),
  kind: normalizeString(raw.kind ?? raw.Kind),
  name: normalizeString(raw.name ?? raw.Name),
  username: normalizeString(raw.username ?? raw.Username),
  scopes: normalizeStringArray(raw.scopes ?? raw.Scopes),
  created_by_user_id: normalizeString(raw.created_by_user_id ?? raw.CreatedByUserID),
  expires_at: normalizeOptionalString(raw.expires_at ?? raw.ExpiresAt),
  revoked_at: normalizeOptionalString(raw.revoked_at ?? raw.RevokedAt),
  last_used_at: normalizeOptionalString(raw.last_used_at ?? raw.LastUsedAt),
  active: normalizeBoolean(raw.active ?? raw.Active),
  created_at: normalizeOptionalString(raw.created_at ?? raw.CreatedAt),
  updated_at: normalizeOptionalString(raw.updated_at ?? raw.UpdatedAt),
});

const normalizeCreatedToken = (payload: unknown): CreatedRepositoryProjectTokenView => {
  const body = resolveBody(payload);
  if (!isRecord(body)) {
    return { project_token: normalizeProjectToken({}), token: "" };
  }
  const tokenRaw = body.project_token ?? body.ProjectToken;
  return {
    project_token: normalizeProjectToken(isRecord(tokenRaw) ? tokenRaw : {}),
    token: normalizeString(body.token ?? body.Token),
  };
};

const normalizeDeployKey = (raw: RawRecord): RepositoryDeployKeyView => ({
  id: normalizeString(raw.id ?? raw.ID),
  project_id: normalizeString(raw.project_id ?? raw.ProjectID),
  title: normalizeString(raw.title ?? raw.Title),
  fingerprint: normalizeString(raw.fingerprint ?? raw.Fingerprint),
  public_key: normalizeString(raw.public_key ?? raw.PublicKey),
  can_push: normalizeBoolean(raw.can_push ?? raw.CanPush),
  created_by_user_id: normalizeString(raw.created_by_user_id ?? raw.CreatedByUserID),
  last_used_at: normalizeOptionalString(raw.last_used_at ?? raw.LastUsedAt),
  created_at: normalizeOptionalString(raw.created_at ?? raw.CreatedAt),
  updated_at: normalizeOptionalString(raw.updated_at ?? raw.UpdatedAt),
});

const dateToRFC3339 = (value: string): string => {
  if (!value) {
    return "";
  }
  return new Date(`${value}T23:59:59Z`).toISOString();
};
