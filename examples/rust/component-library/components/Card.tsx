import React from 'react';

export interface CardProps {
  title: string;
  children: React.ReactNode;
}

export function Card({ title, children }: CardProps) {
  return (
    <div
      style={{
        border: '1px solid #ddd',
        borderRadius: '8px',
        padding: '1.5rem',
        backgroundColor: 'white',
      }}
    >
      <h3
        style={{
          margin: '0 0 1rem 0',
          color: '#333',
          fontSize: '1.25rem',
        }}
      >
        {title}
      </h3>
      <div style={{ color: '#666' }}>{children}</div>
    </div>
  );
}
