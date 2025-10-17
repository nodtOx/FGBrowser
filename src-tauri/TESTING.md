# Testing Guide

## ✅ Current Test Coverage (39 tests)

### Pure Functions - Fully Tested

These are the **easiest to test** because they have:

- ✅ No side effects
- ✅ No I/O operations
- ✅ No database dependencies
- ✅ Deterministic output

#### 1. Size Parsing (`parse_size_to_mb`) - 11 tests

```rust
// Location: src/commands/utils.rs
pub fn parse_size_to_mb(size_str: &Option<String>) -> Option<i64>
```

**Test Coverage:**

- ✅ MB parsing: "916 MB" → 916
- ✅ GB parsing: "1.1 GB" → 1126
- ✅ TB parsing: "1 TB" → 1048576
- ✅ Prefix handling: "from 15.9 GB", "~5 GB"
- ✅ Suffix handling: "15.9 GB [Selective"
- ✅ Slash patterns: "916 MB/1.1 GB" (takes smaller)
- ✅ Same unit slash: "1.1/1.3 GB"
- ✅ Edge cases and invalid input

#### 2. Info Hash Extraction (`extract_info_hash`) - 4 tests

```rust
// Location: src/commands/utils.rs
pub fn extract_info_hash(magnet: &str) -> Option<String>
```

**Test Coverage:**

- ✅ Valid magnet links
- ✅ Multiple parameters
- ✅ Invalid formats
- ✅ Missing btih

#### 3. Blacklist Checking (`is_popular_blacklisted`) - 4 tests

```rust
// Location: src/commands/utils.rs
pub fn is_popular_blacklisted(url: &str) -> bool
```

**Test Coverage:**

- ✅ Contains blacklisted terms
- ✅ Case insensitive matching
- ✅ Not blacklisted URLs
- ✅ Partial matches

#### 4. Title Cleaning (`clean_game_title`) - 20 tests

```rust
// Location: src/crawler/title_cleaner.rs
pub fn clean_game_title(title: &str) -> String
```

**Test Coverage:**

- ✅ Version numbers: "v2.13", "v1.12.3"
- ✅ Build numbers: "Build 12345"
- ✅ Revision numbers: "r34045", ".r49909"
- ✅ Dates: "26.09.2025", "20250831_2044"
- ✅ Editions: "Deluxe Edition", "Ultimate Edition"
- ✅ Platforms: "GOG", "Steam"
- ✅ Parenthetical content: "(Denuvoless)"
- ✅ DLC indicators: "+ 5 DLCs"
- ✅ Complex combinations
- ✅ Real-world examples from FitGirl repacks

---

## 🔄 Next Testing Steps

### Medium Difficulty - Database Service (Mockable)

Now that we have SOLID architecture with dependency injection, these are easier to test:

#### Mock DatabaseService for Commands

```rust
// Example test structure
struct MockDatabaseService;

impl DatabaseService for MockDatabaseService {
    fn search_games(&self, query: &str, limit: i32) -> SqliteResult<Vec<Game>> {
        Ok(vec![
            Game { id: 1, title: "Test Game".to_string(), /* ... */ }
        ])
    }
    // ... other methods
}

#[tokio::test]
async fn test_search_games_command() {
    let mock_service = Arc::new(MockDatabaseService);
    let state = AppState { db_service: mock_service };

    let result = search_games("test".to_string(), 10, State::from(&state)).await;
    assert!(result.is_ok());
}
```

#### Integration Tests with SQLite :memory:

```rust
#[test]
fn test_database_service_integration() {
    let db = Database::new(":memory:".into()).unwrap();
    let service = SqliteDatabaseService::new(db);

    // Test actual database operations
    let games = service.search_games("test", 10).unwrap();
    assert_eq!(games.len(), 0); // Empty database
}
```

### Hard - Integration Tests

These require more setup but are valuable:

#### Crawler Tests

- Test HTML parsing with sample files
- Test error handling
- Test rate limiting

#### Command Integration Tests

- Test full command flow with mock database
- Test error propagation
- Test state management

---

## 📊 Test Statistics

```
Total Tests: 39
✅ Passing: 39 (100%)
❌ Failing: 0
⏭️  Skipped: 0
```

### Coverage by Module

- `commands::utils` - 15 tests (3 functions)
- `crawler::title_cleaner` - 20 tests (1 function)
- `crawler::popular` - 4 tests (existing tests)

---

## 🚀 Running Tests

### Run all tests

```bash
cargo test --lib
```

### Run specific module

```bash
cargo test --lib commands::tests::utils_tests
cargo test --lib commands::tests::title_cleaner_tests
```

### Run with output

```bash
cargo test --lib -- --nocapture
```

### Run in watch mode (with cargo-watch)

```bash
cargo watch -x "test --lib"
```

---

## 💡 Testing Best Practices

1. **Pure Functions First** - These are easiest and provide highest ROI
2. **Use Descriptive Names** - `test_parse_size_with_slash_different_units`
3. **Test Edge Cases** - Empty strings, None, invalid input
4. **Test Real-World Data** - Use actual examples from production
5. **Keep Tests Fast** - Unit tests should run in milliseconds
6. **Mock External Dependencies** - Database, network, filesystem

---

## 🎯 Benefits of Current Test Suite

✅ **Fast Feedback** - All 39 tests run in <0.2 seconds  
✅ **Confidence** - Core utilities are well-tested  
✅ **Regression Prevention** - Changes won't break existing functionality  
✅ **Documentation** - Tests serve as usage examples  
✅ **Refactoring Safety** - Can refactor with confidence

---

## 📝 Test File Structure

```
src/
├── commands/
│   ├── tests/
│   │   ├── mod.rs                    # Test module declaration
│   │   ├── utils_tests.rs            # 15 tests for utils
│   │   └── title_cleaner_tests.rs    # 20 tests for title cleaning
│   ├── utils.rs                      # Pure utility functions
│   └── ...
└── crawler/
    ├── title_cleaner.rs              # Pure title cleaning function
    └── ...
```

---

## 🔍 What Makes These Functions Easy to Test?

### ✅ Pure Functions

```rust
// Input → Function → Output (deterministic)
parse_size_to_mb(&Some("1 GB".to_string())) // Always returns Some(1024)
```

### ❌ Hard to Test (Before SOLID)

```rust
// Multiple side effects, I/O, state changes
#[tauri::command]
pub async fn search_games(state: State) -> Result<Vec<Game>> {
    let db_path = state.db_path.lock().unwrap();  // Lock!
    let db = Database::new(db_path)?;             // I/O!
    db.search_games(query, limit)                 // Database!
}
```

### ✅ Easy to Test (After SOLID)

```rust
// Dependency injection - can mock DatabaseService
#[tauri::command]
pub async fn search_games(state: State<AppState>) -> Result<Vec<Game>> {
    state.db_service.search_games(query, limit)  // Mockable!
}
```

---

## 🎓 Key Takeaways

1. **SOLID = Testable** - Dependency injection makes testing easy
2. **Pure Functions = Free Tests** - No mocking needed
3. **Test Coverage != Quality** - Focus on critical paths
4. **Fast Tests = Happy Devs** - Sub-second test runs encourage TDD

---

_Last Updated: After SOLID refactoring - 39 tests passing ✅_
