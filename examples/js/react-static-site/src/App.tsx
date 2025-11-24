import React, { useState } from 'react';

function App() {
  const [count, setCount] = useState(0);

  return (
    <div style={{
      background: 'white',
      borderRadius: '12px',
      padding: '40px',
      boxShadow: '0 10px 40px rgba(0, 0, 0, 0.2)',
      textAlign: 'center',
    }}>
      <h1 style={{
        fontSize: '2.5rem',
        marginBottom: '20px',
        background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
        WebkitBackgroundClip: 'text',
        WebkitTextFillColor: 'transparent',
        backgroundClip: 'text',
      }}>
        React + Fob
      </h1>
      <p style={{
        fontSize: '1.2rem',
        color: '#666',
        marginBottom: '30px',
      }}>
        A simple React app built with fob CLI
      </p>
      <div style={{
        marginBottom: '30px',
      }}>
        <button
          onClick={() => setCount(count - 1)}
          style={{
            fontSize: '1.5rem',
            width: '50px',
            height: '50px',
            borderRadius: '50%',
            border: 'none',
            background: '#667eea',
            color: 'white',
            cursor: 'pointer',
            marginRight: '20px',
            transition: 'transform 0.2s',
          }}
          onMouseEnter={(e) => e.currentTarget.style.transform = 'scale(1.1)'}
          onMouseLeave={(e) => e.currentTarget.style.transform = 'scale(1)'}
        >
          −
        </button>
        <span style={{
          fontSize: '2rem',
          fontWeight: 'bold',
          color: '#333',
          display: 'inline-block',
          minWidth: '80px',
        }}>
          {count}
        </span>
        <button
          onClick={() => setCount(count + 1)}
          style={{
            fontSize: '1.5rem',
            width: '50px',
            height: '50px',
            borderRadius: '50%',
            border: 'none',
            background: '#764ba2',
            color: 'white',
            cursor: 'pointer',
            marginLeft: '20px',
            transition: 'transform 0.2s',
          }}
          onMouseEnter={(e) => e.currentTarget.style.transform = 'scale(1.1)'}
          onMouseLeave={(e) => e.currentTarget.style.transform = 'scale(1)'}
        >
          +
        </button>
      </div>
      <p style={{
        fontSize: '0.9rem',
        color: '#999',
        fontStyle: 'italic',
      }}>
        Click the buttons to update the counter
      </p>
    </div>
  );
}

export default App;

