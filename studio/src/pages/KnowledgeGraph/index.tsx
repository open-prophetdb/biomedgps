import ChatBox from '@/components/ChatBox';
import { history } from 'umi';
import { Row, Col, Button, message as AntMessage } from 'antd';
import { KnowledgeGraph } from 'biominer-components';
import React, { useEffect, useState, memo } from 'react';
import { useAuth0 } from "@auth0/auth0-react";
import { initChat } from '@/components/util';
// TODO: KeepAlive will cause some bugs, so we disable it for now.
// import { KeepAlive } from 'umi';
import { MessageFilled, MessageOutlined } from '@ant-design/icons';
import {
  fetchEdgesAutoConnectNodes, fetchEntities, fetchEntity2d, fetchEntityColorMap, fetchOneStepLinkedNodes, fetchRelationCounts, fetchStatistics, fetchSubgraphs, fetchPredictedNodes, fetchNodes, fetchRelations, postSubgraph, deleteSubgraph, fetchPaths, askLLM, fetchSharedNodes
} from '@/services/swagger/KnowledgeGraph';
import { getGeneInfo } from '@/plugins4kg/utils';
import { getItems4GenePanel } from '@/plugins4kg';


import './index.less';

const kgFullSpan = 24;
const kgThreeQuartersSpan = 16;

const KnowledgeGraphWithChatBot: React.FC = () => {
  const { isAuthenticated } = useAuth0();
  const [message, setMessage] = useState<string>('')
  const [chatBoxVisible, setChatBoxVisible] = useState<boolean>(false)
  const [span, setSpan] = useState<number>(kgFullSpan)

  useEffect(() => {
    if (!isAuthenticated) {
      history.push('/not-authorized');
    } else {
      // Initialize the chatbot
      initChat();
    }
  }, [isAuthenticated])

  useEffect(() => {
    if (chatBoxVisible) {
      setSpan(kgThreeQuartersSpan)
    } else {
      setSpan(kgFullSpan)
    }
  }, [chatBoxVisible])

  return isAuthenticated ? (
    <Row gutter={8} className="chat-ai-container">
      {
        chatBoxVisible ? (
          <Col xxl={24 - span} xl={24 - span} lg={24 - span} md={24} sm={24} xs={24}>
            <ChatBox message={message}></ChatBox>
          </Col>
        ) : null
      }
      <Col xxl={span} xl={span} lg={span} md={24} sm={24} xs={24}>
        <Button shape="default" className="chat-button" onClick={() => {
          if (chatBoxVisible) {
            // Clear the message when chatbot is closed, otherwise it will activate the chat ai again when chatbot is opened.
            setMessage('')
          }
          setChatBoxVisible(!chatBoxVisible)
        }} icon={chatBoxVisible ? <MessageOutlined /> : <MessageFilled />}>
          {chatBoxVisible ? 'Hide Chatbot' : 'Show Chatbot'}
        </Button>
        <KnowledgeGraph
          apis={{
            GetStatisticsFn: fetchStatistics,
            GetEntitiesFn: fetchEntities,
            GetRelationsFn: fetchRelations,
            GetRelationCountsFn: fetchRelationCounts,
            GetGraphHistoryFn: fetchSubgraphs,
            PostGraphHistoryFn: postSubgraph,
            DeleteGraphHistoryFn: deleteSubgraph,
            GetNodesFn: fetchNodes,
            GetPredictedNodesFn: fetchPredictedNodes,
            GetOneStepLinkedNodesFn: fetchOneStepLinkedNodes,
            GetConnectedNodesFn: fetchEdgesAutoConnectNodes,
            GetEntity2DFn: fetchEntity2d,
            GetEntityColorMapFn: fetchEntityColorMap,
            GetNStepsLinkedNodesFn: fetchPaths,
            AskLlmFn: askLLM,
            GetSharedNodesFn: fetchSharedNodes,
          }}
          getGeneInfo={getGeneInfo}
          getItems4GenePanel={getItems4GenePanel}
          postMessage={(message: string) => {
            if (chatBoxVisible) {
              setMessage(message)
            } else {
              AntMessage.warning('Please open the chatbot first.')
            }
          }}>
        </KnowledgeGraph>
      </Col>
    </Row>
  ) : null
}

export default memo(KnowledgeGraphWithChatBot);
