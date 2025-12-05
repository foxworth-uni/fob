// TSX with React fragments
import React from 'react';

interface ListProps {
  items: string[];
}

export function FragmentList({ items }: ListProps) {
  return (
    <>
      <h2>Items</h2>
      <ul>
        {items.map((item, index) => (
          <li key={index}>{item}</li>
        ))}
      </ul>
    </>
  );
}

export function EmptyFragment() {
  return <></>;
}
