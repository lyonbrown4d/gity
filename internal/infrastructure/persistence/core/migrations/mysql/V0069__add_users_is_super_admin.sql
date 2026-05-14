ALTER TABLE `users` ADD COLUMN `is_super_admin` INTEGER NOT NULL DEFAULT 0 AFTER `email`;

UPDATE `users`
SET `is_super_admin` = 1
WHERE `id` = (
    SELECT `id`
    FROM (
        SELECT `id`
        FROM `users`
        ORDER BY `created_at` ASC, `id` ASC
        LIMIT 1
    ) AS `bootstrap_user`
)
AND NOT EXISTS (
    SELECT 1
    FROM (
        SELECT `id`
        FROM `users`
        WHERE `is_super_admin` = 1
        LIMIT 1
    ) AS `existing_admin`
);
