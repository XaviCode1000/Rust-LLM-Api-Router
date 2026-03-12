# proj-mod-by-feature

**Category:** Project Structure | **Priority:** MEDIUM

Organize modules by feature, not by type.

## Why

Feature-based organization makes it easier to:
- Find related code
- Understand module boundaries
- Add/remove features
- Navigate the codebase

## Examples

### ❌ Bad: Modules by type

```
src/
├── models/
│   ├── user.rs
│   └── post.rs
├── services/
│   ├── user_service.rs
│   └── post_service.rs
├── repositories/
│   ├── user_repo.rs
│   └── post_repo.rs
└── handlers/
    ├── user_handler.rs
    └── post_handler.rs
```

### ✅ Good: Modules by feature

```
src/
├── user/
│   ├── mod.rs
│   ├── entity.rs      # User model
│   ├── service.rs     # User business logic
│   ├── repository.rs  # User persistence
│   └── handler.rs     # User API endpoints
├── post/
│   ├── mod.rs
│   ├── entity.rs
│   ├── service.rs
│   ├── repository.rs
│   └── handler.rs
└── lib.rs
```

## Benefits

- All user-related code in one place
- Easy to delete/move features
- Clear ownership boundaries
- Better for team collaboration

## Related rules

- `proj-lib-main-split`
- `proj-pub-use-reexport`
- `proj-flat-small`
