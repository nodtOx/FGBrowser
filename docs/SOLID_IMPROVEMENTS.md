# SOLID Principles Applied to Commands

## Current State vs. SOLID-Compliant Design

### ❌ Current Issues

```rust
// Repeated in ~50 command functions:
#[tauri::command]
pub async fn search_games(query: String, limit: i32, state: State<'_, AppState>) -> Result<Vec<Game>, String> {
    let db_path = state.db_path.lock().unwrap().clone();  // Lock contention
    let db = Database::new(db_path).map_err(|e| e.to_string())?;  // New connection every call!
    db.search_games(&query, limit).map_err(|e| e.to_string())
}
```

**Problems:**

1. **Violates DRY**: Same 3 lines repeated 50+ times
2. **Performance**: Creates new SQLite connection for every command call
3. **Tight Coupling**: Commands directly depend on concrete `Database` type
4. **Hard to Test**: Can't easily mock database for unit tests
5. **Violates Dependency Inversion**: High-level commands depend on low-level Database implementation

---

## ✅ SOLID-Compliant Solution

### 1. Dependency Inversion Principle (DIP)

Create a `DatabaseService` trait (abstraction):

```rust
// commands/database_service.rs
pub trait DatabaseService: Send + Sync {
    fn search_games(&self, query: &str, limit: i32) -> SqliteResult<Vec<Game>>;
    fn get_all_games(&self, limit: i32, offset: i32) -> SqliteResult<Vec<Game>>;
    // ... other methods
}

// Concrete implementation
pub struct SqliteDatabaseService {
    db: Arc<Mutex<Database>>,  // Single, shared connection
}

impl DatabaseService for SqliteDatabaseService {
    fn search_games(&self, query: &str, limit: i32) -> SqliteResult<Vec<Game>> {
        let db = self.db.lock().unwrap();
        db.search_games(query, limit)
    }
    // ... implementations
}
```

### 2. Dependency Injection

Inject database service into AppState:

```rust
// utils.rs
pub struct AppState {
    pub db_service: Arc<dyn DatabaseService>,  // Depends on abstraction!
}

// lib.rs initialization
let db_service = Arc::new(SqliteDatabaseService::new(db_path)?);
let state = AppState { db_service };
```

### 3. Clean Commands

Commands become simple and focused:

```rust
#[tauri::command]
pub async fn search_games(
    query: String,
    limit: i32,
    state: State<'_, AppState>,
) -> Result<Vec<Game>, String> {
    // Clean, simple, testable!
    state.db_service
        .search_games(&query, limit)
        .map_err(|e| e.to_string())
}
```

---

## Benefits of SOLID Approach

### 🚀 Performance

- **Before**: 50+ database connections per user interaction
- **After**: 1 shared connection (WAL mode allows concurrent reads)
- **Result**: Faster response times, less memory

### 🧪 Testability

```rust
// Easy to create mock for testing
struct MockDatabaseService {
    test_data: Vec<Game>,
}

impl DatabaseService for MockDatabaseService {
    fn search_games(&self, _query: &str, _limit: i32) -> SqliteResult<Vec<Game>> {
        Ok(self.test_data.clone())
    }
}

// Test command without real database
#[cfg(test)]
fn test_search_games() {
    let mock_db = Arc::new(MockDatabaseService { test_data: vec![...] });
    let state = AppState { db_service: mock_db };
    // Test the command...
}
```

### 🔄 Flexibility

Want to switch from SQLite to PostgreSQL? Just implement the trait:

```rust
struct PostgresDatabaseService { /* ... */ }

impl DatabaseService for PostgresDatabaseService {
    fn search_games(&self, query: &str, limit: i32) -> SqliteResult<Vec<Game>> {
        // PostgreSQL implementation
    }
}

// Change one line in main:
let db_service = Arc::new(PostgresDatabaseService::new(config)?);
// All commands work unchanged!
```

### 🧹 Code Quality

- **DRY**: No repeated initialization code
- **Single Responsibility**: Each command does one thing
- **Low Coupling**: Commands don't know about concrete database
- **High Cohesion**: Related operations grouped in service

---

## Comparison

| Aspect                     | Current                  | SOLID           |
| -------------------------- | ------------------------ | --------------- |
| Lines of code              | ~1500                    | ~1200           |
| DB connections per request | 1-5 new                  | 1 shared        |
| Coupling                   | High (concrete Database) | Low (trait)     |
| Testability                | Hard (needs real DB)     | Easy (mockable) |
| Code duplication           | High                     | Minimal         |
| Extensibility              | Limited                  | Excellent       |

---

## Migration Path

If you want to proceed with this refactoring:

1. ✅ Create `DatabaseService` trait (done)
2. ✅ Create `SqliteDatabaseService` implementation (done)
3. ✅ Update `AppState` to use service
4. ✅ Refactor game commands
5. ⏳ Refactor crawler commands
6. ⏳ Refactor popular commands
7. ⏳ Refactor download commands
8. ⏳ Refactor settings commands
9. ⏳ Update `lib.rs` initialization
10. ⏳ Remove old database instantiation pattern

**Estimated Time**: ~2-3 hours to refactor all commands
**Risk**: Low (can do incrementally, keep both patterns working)

---

## Decision

The current module-based refactoring you have is **good enough** for most purposes.

The SOLID improvements add:

- ✅ Better performance (connection reuse)
- ✅ Better testability (mockable services)
- ✅ Better extensibility (easy to swap implementations)
- ❌ More abstractions (slightly more complex)
- ❌ More files (trait + implementation)

**Recommendation**:

- Current refactoring is fine for a working project
- SOLID improvements are worthwhile if:
  - You plan to write unit tests
  - Performance becomes an issue
  - You might switch databases in the future
  - The codebase will grow significantly

Would you like me to complete the SOLID refactoring, or is the current module-based organization sufficient for your needs?
