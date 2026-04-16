# Product Scope Summary

## Core model

### 1. Platform users

Humans who manage the installation.

They can have:

- **platform-level access** to manage the whole server
- **tenant-level access** to manage one or more tenants
- both

### 2. Tenants

A tenant is the main isolation boundary.

A tenant represents:

- an isolated user directory
- isolated auth policies
- isolated admin scope

Most companies will use **one tenant** for their whole ecosystem.  
 Some users may use **multiple tenants** for unrelated projects.

### 3. Tenant users

End-users that belong to a tenant.

These are completely separate from platform users.

### 4. Applications

Applications live inside a tenant and use that tenant’s user base.

This means:

- **tenant = identity/user bucket**
- **application = client of that tenant**

---

## API shape

### Tenant Auth API

This is the main integration surface for SDKs and raw HTTP calls.

Examples:

- `POST /api/v1/tenants/:slug/auth/login`
- `POST /api/v1/tenants/:slug/auth/signup`
- `POST /api/v1/tenants/:slug/auth/logout`
- `POST /api/v1/tenants/:slug/auth/refresh`
- `GET /api/v1/tenants/:slug/auth/me`

Customer apps should interact with the tenant API, not the admin webapp.

### Control-Plane API

Used by the Admin Webapp and operational tooling.

Examples:

- `POST /api/v1/platform/auth/login`
- `GET /api/v1/platform/me`
- `GET /api/v1/platform/tenants`
- `POST /api/v1/platform/tenants`

---

## Webapp scope

The web UI should be treated as an **admin console**.

Suggested direction:

- `/admin/login`
- `/admin/*`
- optional tenant admin pages under:
  - `/admin/tenants/:slug/*`

---

## Main principles

- **Self-hosted first**
- **Multi-tenant by design**
- **Platform users and tenant users are separate**
- **Tenant is the main auth boundary**
- **Admin Webapp wraps the management API**
- **SDK/API is the real integration surface**

---

## Short definition

> A self-hosted multi-tenant auth server where each tenant owns an isolated user directory, applications integrate through an SDK or  
>  HTTP API, and operators manage the system through a bundled admin console.
