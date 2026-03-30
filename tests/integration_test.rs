#[cfg(test)]
mod integration_tests {
    use std::sync::Arc;
    use toto::adapters::storage::sqlite::SqliteRepository;
    use toto::domain::service::TaskService;
    use toto::ports::inbound::TaskServicePort;

    #[test]
    fn test_task_lifecycle_integration() {
        // Use in-memory SQLite for testing
        let repository = Arc::new(SqliteRepository::new_in_memory().unwrap());
        let service = TaskService::new(repository);

        // 1. Add task
        let id = service
            .add_task("Integration test".to_string(), None, None)
            .unwrap();

        // 2. Verify it exists
        let tasks = service.get_all_tasks().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].content, "Integration test");
        assert_eq!(tasks[0].id, id);

        // 3. Toggle completed
        service.toggle_completed(id.clone()).unwrap();
        let tasks = service.get_all_tasks().unwrap();
        assert!(tasks[0].completed);

        // 4. Update content
        service
            .update_task_content(id.clone(), "Updated".to_string(), None, None)
            .unwrap();
        let tasks = service.get_all_tasks().unwrap();
        assert_eq!(tasks[0].content, "Updated");

        // 5. Remove task
        service.remove_task(id).unwrap();
        let tasks = service.get_all_tasks().unwrap();
        assert_eq!(tasks.len(), 0);
    }
}
