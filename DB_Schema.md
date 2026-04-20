# Simple Multi-Tenant Auth Schema

## Overview

- `platform_users` are humans who can log into the system dashboard.
- `tenants` are customer buckets like Meta or OpenAI.
- `tenant_users` are the end users who log into the tenant's webapps.

## `platform_users`

Platform level users that manages the various tenants and the platform itself

| Column          | Type        | Notes                            |
| --------------- | ----------- | -------------------------------- |
| `id`            | uuid        | PK                               |
| `email`         | text        | unique across the whole platform |
| `password_hash` | text        | hashed only                      |
| `role`          | enum        | `owner`, `admin`, `user`         |
| `status`        | enum        | `active`, `disabled`, `locked`   |
| `created_at`    | timestamptz | default now                      |
| `updated_at`    | timestamptz | default now                      |

## `tenants`

User bucket basically

| Column       | Type        | Notes                             |
| ------------ | ----------- | --------------------------------- |
| `id`         | uuid        | PK                                |
| `slug`       | text        | unique, human-readable tenant key |
| `name`       | text        | display name                      |
| `status`     | enum        | `active`, `suspended`, `deleted`  |
| `created_at` | timestamptz | default now                       |
| `updated_at` | timestamptz | default now                       |

## `tenant_members`

Platform users assigned to tenants.

| Column             | Type        | Notes                     |
| ------------------ | ----------- | ------------------------- |
| `tenant_id`        | uuid        | FK -> `tenants.id`        |
| `platform_user_id` | uuid        | FK -> `platform_users.id` |
| `created_at`       | timestamptz | default now               |

NOTE: a platform user can belong to multiple tenants

## `tenant_users`

End users who register and log into the tenant's apps.
(need to work out a better schema, here for future reference)

| Column               | Type        | Notes                          |
| -------------------- | ----------- | ------------------------------ |
| `id`                 | uuid        | PK                             |
| `tenant_id`          | uuid        | FK -> `tenants.id`             |
| `email`              | text        | raw email                      |
| `email_normalized`   | text        | unique within tenant           |
| `password_hash`      | text        | hashed only                    |
| `password_hash_algo` | text        | hash algorithm name            |
| `display_name`       | text        | nullable                       |
| `status`             | enum        | `active`, `disabled`, `locked` |
| `email_verified_at`  | timestamptz | nullable                       |
| `last_login_at`      | timestamptz | nullable                       |
| `created_at`         | timestamptz | default now                    |
| `updated_at`         | timestamptz | default now                    |

## `applications`

Webapps owned by a tenant.
(not sure if it makes sense to have this, maybe for API calls validation so WebAppX of TenantX cannot make requests to TenantY)

| Column       | Type        | Notes                                     |
| ------------ | ----------- | ----------------------------------------- |
| `id`         | uuid        | PK                                        |
| `tenant_id`  | uuid        | FK -> `tenants.id`                        |
| `name`       | text        | app name                                  |
| `client_id`  | text        | public client identifier, unique globally |
| `status`     | enum        | `active`, `suspended`, `deleted`          |
| `created_at` | timestamptz | default now                               |
| `updated_at` | timestamptz | default now                               |
