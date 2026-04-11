#[cfg(test)]
mod integration_tests {
    use std::sync::Arc;
    use toto::adapters::storage::sqlite::SqliteRepository;
    use toto::domain::service::TaskService;
    use toto::domain::task::RelationType;
    use toto::ports::inbound::TaskServicePort;

    #[test]
    fn test_task_lifecycle_integration() {
        // Use in-memory SQLite for testing
        let repository = Arc::new(SqliteRepository::new_in_memory().unwrap());
        let service = TaskService::new(repository);

        // 1. Add task
        let id = service
            .add_task(
                "Integration test".to_string(),
                "Description".to_string(),
                None,
                None,
            )
            .unwrap();

        // 2. Verify it exists
        let tasks = service.get_all_tasks().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title(), "Integration test");
        assert_eq!(tasks[0].description(), "Description");
        assert_eq!(tasks[0].id, id);

        // 3. Toggle completed
        service.toggle_completed(id.clone()).unwrap();
        let tasks = service.get_all_tasks().unwrap();
        assert!(tasks[0].is_completed());

        // 4. Update content
        service
            .update_task(
                id.clone(),
                "Updated".to_string(),
                "New description".to_string(),
                None,
                None,
            )
            .unwrap();
        let tasks = service.get_all_tasks().unwrap();
        assert_eq!(tasks[0].title(), "Updated");
        assert_eq!(tasks[0].description(), "New description");

        // 5. Tags
        service.add_tag(id.clone(), "rust".to_string()).unwrap();
        service.add_tag(id.clone(), "cli".to_string()).unwrap();
        let tasks = service.get_all_tasks().unwrap();
        assert_eq!(tasks[0].tags(), vec!["rust", "cli"]);

        service.remove_tag(id.clone(), "rust".to_string()).unwrap();
        let tasks = service.get_all_tasks().unwrap();
        assert_eq!(tasks[0].tags(), vec!["cli"]);

        // 6. Relations
        let id2 = service
            .add_task(
                "Task 2".to_string(),
                "Description 2".to_string(),
                None,
                None,
            )
            .unwrap();

        service
            .add_relation(id.clone(), id2.clone(), RelationType::Blocks)
            .unwrap();

        let tasks = service.get_all_tasks().unwrap();
        let task1 = tasks.iter().find(|t| t.id == id).unwrap();
        let task2 = tasks.iter().find(|t| t.id == id2).unwrap();

        assert_eq!(task1.relations()[0].target_id, id2);
        assert_eq!(task1.relations()[0].relation_type, RelationType::Blocks);

        assert_eq!(task2.relations()[0].target_id, id);
        assert_eq!(task2.relations()[0].relation_type, RelationType::BlockedBy);

        // 7. Remove task
        service.remove_task(id).unwrap();
        let tasks = service.get_all_tasks().unwrap();
        assert_eq!(tasks.len(), 1);
    }
}
