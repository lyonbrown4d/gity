import { expect, type APIRequestContext, type APIResponse, type Page } from "@playwright/test";

const apiBaseURL = process.env.E2E_API_BASE_URL ?? "http://127.0.0.1:19083/api/v1";

type Request = APIRequestContext;

interface Envelope<T> {
  code: number;
  message: string;
  data: T;
}

export interface AuthSession {
  user_id: string;
  username: string;
  token: string;
  refresh_token: string;
}

export interface OrganizationView {
  id: string;
  key: string;
  name: string;
}

export interface ProjectView {
  id: string;
  organization_id: string;
  key: string;
  full_path: string;
  name: string;
  default_branch: string;
}

export interface BranchView {
  name: string;
  is_default: boolean;
  hash: string;
}

export interface IssueView {
  id: string;
  number: number;
  title: string;
}

export const uniqueKey = (prefix: string): string =>
  `${prefix}-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 7)}`;

export const loginByAPI = async (request: Request, username = uniqueKey("e2e-user")): Promise<AuthSession> => {
  const response = await request.post(`${apiBaseURL}/auth/login`, {
    data: {
      username,
      password: "password",
    },
  });
  return responseData<AuthSession>(response);
};

export const authenticatePage = async (page: Page, session: AuthSession): Promise<void> => {
  await page.goto("/");
  await page.evaluate(
    ({ accessToken, refreshToken }) => {
      localStorage.setItem("gity.access_token", accessToken);
      localStorage.setItem("gity.refresh_token", refreshToken);
    },
    { accessToken: session.token, refreshToken: session.refresh_token },
  );
};

export const createOrganization = async (request: Request, session: AuthSession, key = uniqueKey("org")): Promise<OrganizationView> => {
  const response = await request.post(`${apiBaseURL}/orgs`, {
    headers: authHeaders(session),
    data: {
      key,
      path_key: key,
      owner_user_id: Number(session.user_id),
      name: `E2E ${key}`,
      description: "Created by Playwright integration tests",
      visibility: "private",
    },
  });
  return responseData<OrganizationView>(response);
};

export const createProject = async (
  request: Request,
  session: AuthSession,
  organization: OrganizationView,
  key = uniqueKey("project"),
): Promise<ProjectView> => {
  const response = await request.post(`${apiBaseURL}/projects`, {
    headers: authHeaders(session),
    data: {
      organization_id: organization.id,
      key,
      path_key: key,
      name: `E2E ${key}`,
      description: "Playwright seeded project",
      visibility: "private",
      default_branch: "main",
    },
  });
  return responseData<ProjectView>(response);
};

export const createBranch = async (
  request: Request,
  session: AuthSession,
  project: ProjectView,
  name: string,
  sourceRef = project.default_branch || "main",
): Promise<BranchView> => {
  const response = await request.post(`${apiBaseURL}/projects/${project.id}/repository/branches`, {
    headers: authHeaders(session),
    data: {
      name,
      source_ref: sourceRef,
    },
  });
  return responseData<BranchView>(response);
};

export const createFileCommit = async (
  request: Request,
  session: AuthSession,
  project: ProjectView,
  options: {
    branchName?: string;
    path?: string;
    content?: string;
    message?: string;
  } = {},
): Promise<void> => {
  const filePath = options.path ?? "README.md";
  const response = await request.post(`${apiBaseURL}/projects/${project.id}/repository/files`, {
    headers: authHeaders(session),
    data: {
      branch_name: (options.branchName ?? project.default_branch) || "main",
      path: filePath,
      content: options.content ?? `# ${project.name}\n\nCreated by Playwright integration tests.\n`,
      message: options.message ?? `Add ${filePath} from Playwright`,
      author_name: session.username,
      author_email: `${session.username}@local.gity`,
    },
  });
  await expectOK(response);
};

export const createIssue = async (request: Request, session: AuthSession, project: ProjectView): Promise<IssueView> => {
  const issueTitle = `E2E issue ${uniqueKey("case")}`;
  const response = await request.post(`${apiBaseURL}/projects/${project.id}/issues`, {
    headers: authHeaders(session),
    data: {
      title: issueTitle,
      description: "Issue seeded for UI integration testing",
    },
  });
  return responseData<IssueView>(response);
};

export const createIssueComment = async (
  request: Request,
  session: AuthSession,
  project: ProjectView,
  issueNumber: number,
  content: string,
): Promise<void> => {
  const response = await request.post(`${apiBaseURL}/projects/${project.id}/issues/${issueNumber}/comments`, {
    headers: authHeaders(session),
    data: {
      body: content,
      content,
    },
  });
  await expectOK(response);
};

export const seedProject = async (request: Request): Promise<{
  session: AuthSession;
  organization: OrganizationView;
  project: ProjectView;
  issue: IssueView;
}> => {
  const session = await loginByAPI(request);
  const organization = await createOrganization(request, session);
  const project = await createProject(request, session, organization);
  await createFileCommit(request, session, project);
  const issue = await createIssue(request, session, project);
  return { session, organization, project, issue };
};

const authHeaders = (session: AuthSession): Record<string, string> => ({
    Authorization: `Bearer ${session.token}`,
  });

const responseData = async <T>(response: APIResponse): Promise<T> => {
  const body = await expectOK(response);
  try {
    const payload = JSON.parse(body) as T | Envelope<T>;
    if (isEnvelope<T>(payload)) {
      return payload.data;
    }
    return payload;
  } catch (error) {
    throw new Error(`${response.url()} returned non-JSON response: ${body.slice(0, 200)}`, { cause: error });
  }
};

const expectOK = async (response: APIResponse): Promise<string> => {
  const body = await response.text();
  expect(response.ok(), `${response.status()} ${response.url()} failed: ${body}`).toBe(true);
  return body;
};

const isEnvelope = <T>(value: T | Envelope<T>): value is Envelope<T> => typeof value === "object" && value !== null && "data" in value && "code" in value;


