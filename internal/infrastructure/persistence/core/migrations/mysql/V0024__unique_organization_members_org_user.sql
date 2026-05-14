CREATE UNIQUE INDEX `ux_organization_members_org_user` ON `organization_members` (`organization_id`, `user_id`);
