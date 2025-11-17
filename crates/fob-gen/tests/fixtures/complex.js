// Complex JavaScript with various features
import React, { useState, useEffect } from 'react';
import { debounce } from './utils';

async function fetchData(url) {
    try {
        const response = await fetch(url);
        const data = await response.json();
        return data;
    } catch (error) {
        console.error('Failed to fetch:', error);
        throw error;
    }
}

const Component = () => {
    const [count, setCount] = useState(0);
    const [data, setData] = useState(null);

    useEffect(() => {
        const loadData = debounce(async () => {
            const result = await fetchData('/api/data');
            setData(result);
        }, 300);
        
        loadData();
    }, [count]);

    const handleClick = () => {
        setCount(prev => prev + 1);
    };

    return (
        <div>
            <p>Count: {count}</p>
            <button onClick={handleClick}>Increment</button>
            {data && <pre>{JSON.stringify(data, null, 2)}</pre>}
        </div>
    );
};

export default Component;

