// JSX elements and attributes
import React from 'react';

function Button({ label, onClick, className = "btn" }) {
    return (
        <button onClick={onClick} className={className}>
            {label}
        </button>
    );
}

const App = () => (
    <div className="container">
        <h1>Hello World</h1>
        <Button label="Click me" onClick={() => alert('clicked')} />
    </div>
);

export default App;

