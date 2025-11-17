/**
 * Description of a single change in the editor
 */
export interface ChangeDescription {
  /** Start offset of the change */
  from: number;
  /** End offset of the change */
  to: number;
  /** Text that was inserted */
  inserted: string;
  /** Text that was deleted */
  deleted: string;
}

/**
 * Result of an operation that modifies the editor
 * Every mutation returns this for observability
 */
export interface OperationResult {
  /** Whether the operation succeeded */
  success: boolean;
  /** Array of changes made */
  changes: ChangeDescription[];
  /** Error message if operation failed */
  error?: string;
}
