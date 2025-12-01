/**
 * Hook for accessing task list functionality in MDX content
 */

import { useTaskListContext, type TaskListState } from './TaskListContext.js';

export interface UseTaskListReturn {
  /**
   * Current state of all tasks
   */
  taskState: TaskListState;

  /**
   * Toggle a task's checked state
   */
  toggleTask: (taskId: string, checked: boolean) => void;

  /**
   * Check if task list features are available
   */
  isEnabled: boolean;
}

/**
 * Hook to access task list state and controls
 * Returns task state and toggle function if TaskListProvider is present
 *
 * @example
 * ```tsx
 * function CustomCheckbox({ taskId }: { taskId: string }) {
 *   const { taskState, toggleTask, isEnabled } = useTaskList();
 *
 *   if (!isEnabled) return null;
 *
 *   return (
 *     <input
 *       type="checkbox"
 *       checked={taskState[taskId] || false}
 *       onChange={(e) => toggleTask(taskId, e.target.checked)}
 *     />
 *   );
 * }
 * ```
 */
export function useTaskList(): UseTaskListReturn {
  const context = useTaskListContext();

  if (!context) {
    // Return no-op implementation when provider is not present
    return {
      taskState: {},
      toggleTask: () => {},
      isEnabled: false,
    };
  }

  return {
    taskState: context.taskState,
    toggleTask: context.toggleTask,
    isEnabled: true,
  };
}
