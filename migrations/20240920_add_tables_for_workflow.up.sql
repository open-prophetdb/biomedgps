-- This file is used to create the tables for the workspace, workflow, task, and notification.

CREATE TABLE
    IF NOT EXISTS biomedgps_workspace (
        id VARCHAR(36) PRIMARY KEY,
        workspace_name VARCHAR(64),
        description TEXT,
        created_time TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_time TIMESTAMPTZ NOT NULL,
        archived_time TIMESTAMPTZ,
        payload JSONB NOT NULL, -- Any additional information for the workspace.
        owner VARCHAR(32) NOT NULL,
        groups VARCHAR(32)[] NOT NULL,

        CONSTRAINT biomedgps_workspace_uniq_key UNIQUE (workspace_name)
    );

-- The workflow table is used to store the information of workflows which are installed in the system.
-- Example:
-- {
--   "name": "BarPlot",
--   "version": "v0.1.0",
--   "description": "",
--   "category": "Chart",
--   "home": "https://github.com/rapex-lab/rapex/tree/master/rapex/src/rapex/tasks",
--   "source": "Rapex Team",
--   "short_name": "chart-name",
--   "icons": [
--     {
--       "src": "",
--       "type": "image/png",
--       "sizes": "144x144"
--     }
--   ],
--   "author": "Jingcheng Yang",
--   "maintainers": [
--     "Jingcheng Yang",
--     "Tianyuan Cheng"
--   ],
--   "tags": [
--     "R",
--     "Chart"
--   ],
--   "readme": "https://rapex.prophetdb.org/README/barplot.md",
--   "id": "chart-name"
-- }
CREATE TABLE
    IF NOT EXISTS biomedgps_workflow (
        id VARCHAR(32) PRIMARY KEY,
        name VARCHAR(255),
        version VARCHAR(255),
        description TEXT,
        category VARCHAR(255),
        home TEXT,
        source VARCHAR(255),
        short_name VARCHAR(255),
        icons JSONB,
        author VARCHAR(64),
        maintainers VARCHAR(255)[],
        tags VARCHAR(255)[],
        readme TEXT
    );

CREATE TABLE
    IF NOT EXISTS biomedgps_task (
        id VARCHAR(36) PRIMARY KEY,
        workspace_id VARCHAR(36) NOT NULL, -- One workspace has many tasks.
        workflow_id VARCHAR(36) NOT NULL, -- One workflow has many tasks.
        task_id VARCHAR(36), -- One task has one task_id. We need to generate an uuid for tracking the task from the cromwell server.
        task_name VARCHAR(32) NOT NULL,
        description TEXT,
        submitted_time TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
        started_time TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
        finished_time TIMESTAMPTZ,
        task_params JSONB NOT NULL,
        labels JSONB NOT NULL,
        status VARCHAR(32) NOT NULL, -- The status of the task, such as running, finished, failed, etc.
        owner VARCHAR(32) NOT NULL,
        groups VARCHAR(32)[] NOT NULL,
        CONSTRAINT biomedgps_workflow_uniq_key UNIQUE (workspace_id, task_id)
    );

CREATE TABLE
    IF NOT EXISTS biomedgps_notification (
        id SERIAL NOT NULL,
        title VARCHAR(255) NOT NULL,
        description TEXT,
        notification_type VARCHAR(32) NOT NULL,
        created_time BIGINT NOT NULL,
        status VARCHAR(32) NOT NULL,
        owner VARCHAR(32) NOT NULL
    );