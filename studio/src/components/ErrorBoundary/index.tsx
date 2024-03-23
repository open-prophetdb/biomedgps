import React from 'react';
import { history } from 'umi';

class ErrorBoundary extends React.Component {
    state = { hasError: false };

    static getDerivedStateFromError(error: any) {
        // Update state so the next render will show the fallback UI.
        return { hasError: true };
    }

    componentDidCatch(error: any, errorInfo: any) {
        // You can log the error to an error reporting service
        console.log(error, errorInfo);
    }

    render() {
        if (this.state.hasError) {
            history.push('/404');
        }

        // @ts-ignore
        return this.props.children;
    }
};

export default ErrorBoundary;