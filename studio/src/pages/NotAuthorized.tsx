import { Button, Result, Row } from 'antd';
import { useAuth0 } from "@auth0/auth0-react";
import React from 'react';

const NotAuthorizedPage: React.FC = () => {
  const { loginWithRedirect } = useAuth0();

  return <Result
    style={{
      width: '50%',
      justifyContent: 'center',
      display: 'flex',
      flexDirection: 'column',
      alignItems: 'center',
      margin: 'auto'
    }}
    status="403"
    title="Not Authorized"
    subTitle="Sorry, you are not authorized to access this page. If you have an account on prophet-studio.3steps.cn or this website, please login first. You can use your google, github or microsoft account or sign up a new account at the sign-in page. If you have any questions, please click the `Contact Us` button for the contact information."
    extra={
      <Row>
        <Button onClick={() => {
          loginWithRedirect()
        }}>
          Sign In / Sign Up
        </Button>
        <Button style={{ marginLeft: '5px' }} onClick={() => {
          window.open("https://www.prophetdb.org/contact/", "_blank")
        }}>
          Contact Us
        </Button>
      </Row>
    }
  />
};

export default NotAuthorizedPage;
