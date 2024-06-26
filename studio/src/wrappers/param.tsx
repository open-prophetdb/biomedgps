import { Navigate, Outlet } from 'umi'

export default (props: any) => {
    return <Navigate to={`/predict-explain/knowledge-table?nodeIds=Disease::MONDO:0005404,Disease::MONDO:0100233`} />;
}