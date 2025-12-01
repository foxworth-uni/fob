'use client';

/**
 * Task List Context for interactive MDX task lists
 * Provides state management and toggle handling for task list checkboxes
 */

import { createContext, useContext, useState, useCallback, type ReactNode } from 'react';

export interface TaskListState {
  [taskId: string]: boolean;
}

export interface TaskListContextValue {
  taskState: TaskListState;
  toggleTask: (taskId: string, checked: boolean) => void;
}

const TaskListContext = createContext<TaskListContextValue | null>(null);

if (process.env.NODE_ENV !== 'production') {
  TaskListContext.displayName = 'TaskListContext';
}

export interface TaskListProviderProps {
  children?: ReactNode;
  initialState?: TaskListState;
  onTaskToggle?: (taskId: string, checked: boolean, allState: TaskListState) => void;
}

/**
 * Provider for task list state management
 * Wrap your MDX content with this to enable interactive task lists
 *
 * @example
 * ```tsx
 * <TaskListProvider onTaskToggle={(id, checked) => console.log(id, checked)}>
 *   <MDXContent />
 * </TaskListProvider>
 * ```
 */
export function TaskListProvider({
  children,
  initialState = {},
  onTaskToggle,
}: TaskListProviderProps) {
  const [taskState, setTaskState] = useState<TaskListState>(initialState);

  const toggleTask = useCallback(
    (taskId: string, checked: boolean) => {
      setTaskState((prev) => {
        const newState = { ...prev, [taskId]: checked };
        onTaskToggle?.(taskId, checked, newState);
        return newState;
      });
    },
    [onTaskToggle]
  );

  const value: TaskListContextValue = {
    taskState,
    toggleTask,
  };

  return <TaskListContext.Provider value={value}>{children}</TaskListContext.Provider>;
}

/**
 * Hook to access task list context
 * @internal Used by generated MDX code
 */
export function useTaskListContext(): TaskListContextValue | null {
  return useContext(TaskListContext);
}
