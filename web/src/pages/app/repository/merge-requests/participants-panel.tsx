import { RefreshCw, UserRound } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Section } from "@/components/surface/section";
import { Select, SelectContent, SelectGroup, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import type { RepositoryMergeRequestParticipantsView, UserView } from "@/pages/types";
import { formatUserLabel } from "../repository-user-utils";

export const MergeRequestParticipantsPanel = ({
  reviewers,
  assignees,
  users,
  userByID,
  reviewerDraftUserID,
  assigneeDraftUserID,
  isLoading,
  isUpdating,
  canEdit,
  t,
  onChangeReviewerDraftUserID,
  onChangeAssigneeDraftUserID,
  onAddReviewer,
  onRemoveReviewer,
  onAddAssignee,
  onRemoveAssignee,
  onReload,
}: {
  reviewers: RepositoryMergeRequestParticipantsView["participants"];
  assignees: RepositoryMergeRequestParticipantsView["participants"];
  users: UserView[];
  userByID: Map<string, UserView>;
  reviewerDraftUserID: string;
  assigneeDraftUserID: string;
  isLoading: boolean;
  isUpdating: boolean;
  canEdit: boolean;
  t: (text: string) => string;
  onChangeReviewerDraftUserID: (value: string) => void;
  onChangeAssigneeDraftUserID: (value: string) => void;
  onAddReviewer: () => void;
  onRemoveReviewer: (userID: string) => void;
  onAddAssignee: () => void;
  onRemoveAssignee: (userID: string) => void;
  onReload: () => void;
}) => (
  <Section
    title={
      <span className="flex items-center gap-2">
        <UserRound className="size-4" aria-hidden="true" />
        {t("Reviewers and assignees")}
      </span>
    }
    description={t("Assign people who should review or own this merge request.")}
    actions={
      <Button type="button" size="sm" variant="ghost" onClick={onReload} disabled={isLoading}>
        <RefreshCw data-icon="inline-start" />
        {isLoading ? t("Loading...") : t("Reload")}
      </Button>
    }
    flush
    className="overflow-hidden"
  >
    <div className="grid gap-2 px-4 py-2.5">
      <ParticipantRoleSection
        label={t("Reviewers")}
        emptyLabel={t("No reviewers assigned.")}
        selected={reviewers}
        users={users}
        userByID={userByID}
        draftUserID={reviewerDraftUserID}
        isUpdating={isUpdating}
        canEdit={canEdit}
        t={t}
        onChangeDraftUserID={onChangeReviewerDraftUserID}
        onAdd={onAddReviewer}
        onRemove={onRemoveReviewer}
      />
      <ParticipantRoleSection
        label={t("Assignees")}
        emptyLabel={t("No assignees assigned.")}
        selected={assignees}
        users={users}
        userByID={userByID}
        draftUserID={assigneeDraftUserID}
        isUpdating={isUpdating}
        canEdit={canEdit}
        t={t}
        onChangeDraftUserID={onChangeAssigneeDraftUserID}
        onAdd={onAddAssignee}
        onRemove={onRemoveAssignee}
      />
    </div>
  </Section>
);

const ParticipantRoleSection = ({
  label,
  emptyLabel,
  selected,
  users,
  userByID,
  draftUserID,
  isUpdating,
  canEdit,
  t,
  onChangeDraftUserID,
  onAdd,
  onRemove,
}: {
  label: string;
  emptyLabel: string;
  selected: RepositoryMergeRequestParticipantsView["participants"];
  users: UserView[];
  userByID: Map<string, UserView>;
  draftUserID: string;
  isUpdating: boolean;
  canEdit: boolean;
  t: (text: string) => string;
  onChangeDraftUserID: (value: string) => void;
  onAdd: () => void;
  onRemove: (userID: string) => void;
}) => {
  const selectedIDs = selected.map((item) => item.user_id);
  const availableUsers = users.filter((user) => !selectedIDs.includes(user.id));
  return (
    <div className="flex flex-col gap-1.5">
      <p className="text-xs font-medium text-muted-foreground">{label}</p>
      <div className="flex flex-wrap gap-2">
        {selected.length === 0 ? <span className="text-xs text-muted-foreground">{emptyLabel}</span> : null}
        {selected.map((item) => (
          <Badge key={`${item.role}-${item.user_id}`} variant="outline" className="gap-2">
            {formatUserLabel(userByID.get(item.user_id), item.user_id)}
            <Button
              type="button"
              size="sm"
              variant="ghost"
              className="h-4 px-1 text-muted-foreground hover:text-foreground"
              disabled={!canEdit || isUpdating}
              aria-label={`${t("Remove")} ${formatUserLabel(userByID.get(item.user_id), item.user_id)}`}
              onClick={() => onRemove(item.user_id)}
            >
              x
            </Button>
          </Badge>
        ))}
      </div>
      <div className="grid gap-2 grid-cols-[minmax(0,1fr)_auto]">
        <Select value={draftUserID} onValueChange={onChangeDraftUserID} disabled={!canEdit || isUpdating || availableUsers.length === 0}>
          <SelectTrigger className="min-w-0">
            <SelectValue placeholder={availableUsers.length === 0 ? t("No users available") : t("Select user")} />
          </SelectTrigger>
          <SelectContent>
            <SelectGroup>
              {availableUsers.map((user) => (
                <SelectItem key={user.id} value={user.id}>
                  {formatUserLabel(user, user.id)}
                </SelectItem>
              ))}
            </SelectGroup>
          </SelectContent>
        </Select>
        <Button type="button" size="sm" variant="outline" disabled={!canEdit || isUpdating || !draftUserID} onClick={onAdd}>
          {isUpdating ? t("Saving...") : t("Add")}
        </Button>
      </div>
    </div>
  );
};
