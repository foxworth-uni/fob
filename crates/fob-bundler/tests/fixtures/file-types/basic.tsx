// Basic TSX test fixture - React component with TypeScript
import React from 'react';

interface ButtonProps {
  label: string;
  onClick?: () => void;
}

export function Button({ label, onClick }: ButtonProps) {
  return (
    <button className="btn" onClick={onClick}>
      {label}
    </button>
  );
}

export default Button;
