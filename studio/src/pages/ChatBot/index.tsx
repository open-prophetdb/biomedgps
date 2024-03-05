import ChatBox from '@/components/ChatBox';
import { Row } from 'antd';
import './index.less';

const ChatAI: React.FC = () => {
  return <Row className="chat-ai-container">
    <ChatBox></ChatBox>
  </Row>;
}

export default ChatAI;
