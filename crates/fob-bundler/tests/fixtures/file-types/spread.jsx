// JSX with spread props
import React from 'react';

export function Wrapper(props) {
  return <div className="wrapper" {...props} />;
}

export function ForwardedButton({ children, ...rest }) {
  return (
    <button type="button" {...rest}>
      {children}
    </button>
  );
}

export default function SpreadExample() {
  const sharedProps = {
    className: 'shared',
    'data-testid': 'test-element',
  };

  return (
    <Wrapper {...sharedProps}>
      <ForwardedButton onClick={() => console.log('clicked')} {...sharedProps}>
        Click me
      </ForwardedButton>
    </Wrapper>
  );
}
