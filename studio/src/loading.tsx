import { Spin } from "antd";

export default () => {
  return (
    <div style={{
      display: "flex",
      flexDirection: "column",
      alignItems: "center",
      justifyContent: "center",
      height: "100%",
      minHeight: "420px",
    }} >
      <div style={{ flexDirection: "column", display: 'flex' }}>
        <img src={require('@/assets/logo-white.png')} alt="logo" width="256" />
        <Spin size="large" />
      </div>
    </div>
  );
}