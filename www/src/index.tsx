import React from 'react';
import ReactDOM from 'react-dom/client';
import './index.css';
import App from './App';

const anyWindow = (window as any);
if (!anyWindow.refreshInterval) {
  anyWindow.refreshInterval = setInterval(
    () => {
      anyWindow.globalRefreshHook();
    }, 500
  )
}

const root = ReactDOM.createRoot(
  document.getElementById('root') as HTMLElement
);
root.render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
