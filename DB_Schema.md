# Simple Multi-Tenant Auth Schema

## Overview

- `platform_users` are humans who can log into the system dashboard.
- `tenants` are customer buckets like Meta or OpenAI.
- `tenant_members` assigns platform users to tenants with a role.
- `tenant_users` are the end users who log into the tenant's webapps.

## `platform_users`

Humans with access to the platform dashboard.

| Column          | Type        | Notes                            |
| --------------- | ----------- | -------------------------------- |
| `id`            | uuid        | PK                               |
| `email`         | text        | unique across the whole platform |
| `password_hash` | text        | hashed only                      |
| `password_hash_algo` | text    | hash algorithm name              |
| `display_name`  | text        | nullable                         |
| `status`        | enum        | `active`, `disabled`, `locked`   |
| `created_at`    | timestamptz | default now                      |
| `updated_at`    | timestamptz | default now                      |

## `tenants`

Customer organizations like Meta or OpenAI.

| Column       | Type        | Notes                             |
| ------------ | ----------- | --------------------------------- |
| `id`         | uuid        | PK                                |
| `slug`       | text        | unique, human-readable tenant key |
| `name`       | text        | display name                      |
| `status`     | enum        | `active`, `suspended`, `deleted`  |
| `created_at` | timestamptz | default now                       |
| `updated_at` | timestamptz | default now                       |

## `tenant_members`

Platform users assigned to customer tenants.

| Column             | Type        | Notes                     |
| ------------------ | ----------- | ------------------------- |
| `tenant_id`        | uuid        | FK -> `tenants.id`        |
| `platform_user_id` | uuid        | FK -> `platform_users.id` |
| `role`             | enum        | `owner`, `admin`          |
| `created_at`       | timestamptz | default now               |

Notes:

- each tenant must have exactly one `owner`
- a platform user can belong to multiple tenants
- `owner` is just a role, not a separate table

## `tenant_users`

End users who register and log into the tenant's apps.

| Column              | Type        | Notes                          |
| ------------------- | ----------- | ------------------------------ |
| `id`                | uuid        | PK                             |
| `tenant_id`         | uuid        | FK -> `tenants.id`             |
| `email`             | text        | raw email                      |
| `email_normalized`  | text        | unique within tenant           |
| `password_hash`     | text        | hashed only                    |
| `password_hash_algo`| text        | hash algorithm name            |
| `display_name`      | text        | nullable                       |
| `status`            | enum        | `active`, `disabled`, `locked` |
| `email_verified_at` | timestamptz | nullable                       |
| `last_login_at`     | timestamptz | nullable                       |
| `created_at`        | timestamptz | default now                    |
| `updated_at`        | timestamptz | default now                    |

## `applications`

Webapps owned by a tenant.

| Column       | Type        | Notes                                     |
| ------------ | ----------- | ----------------------------------------- |
| `id`         | uuid        | PK                                        |
| `tenant_id`  | uuid        | FK -> `tenants.id`                        |
| `name`       | text        | app name                                  |
| `client_id`  | text        | public client identifier, unique globally |
| `status`     | enum        | `active`, `suspended`, `deleted`          |
| `created_at` | timestamptz | default now                               |
| `updated_at` | timestamptz | default now                               |

## `application_redirect_uris`

Allowed redirect URIs for each application.

| Column           | Type        | Notes                   |
| ---------------- | ----------- | ----------------------- |
| `id`             | uuid        | PK                      |
| `application_id` | uuid        | FK -> `applications.id` |
| `uri`            | text        | exact redirect URI      |
| `created_at`     | timestamptz | default now             |

## Key Constraints

- `platform_users.email` must be unique.
- `tenants.slug` must be unique.
- `tenant_members` should use `unique(tenant_id, platform_user_id)`.
- `tenant_users` should use `unique(tenant_id, email_normalized)`.
- `applications.client_id` should be unique.
- `application_redirect_uris` should be `unique(application_id, uri)`.

## Design Notes

- Keep platform identity separate from tenant app users.
- Store secrets only as hashes.
- Use `tenant_members` for tenant ownership and admin access.
- Keep tenant app users isolated by `tenant_id`.
