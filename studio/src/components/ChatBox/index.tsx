import { ReactChatPlugin } from 'biominer-components';
import { filter, set } from 'lodash';
import * as webllm from "@mlc-ai/web-llm";
import { initChat } from '@/components/util';
import { useEffect, useState } from 'react';
import { message as AntdMessage } from 'antd';
import rehypeRaw from 'rehype-raw';
import rehypeHighlight from 'rehype-highlight';

import './index.less';

interface Author {
  id: number
  username?: string
  avatarUrl?: string
}

interface Button {
  type: string;
  title: string;
  payload: string;
}

export interface Message {
  key?: string;
  text: string;
  timestamp: number;
  type: string;
  author?: Author;
  buttons?: Button[];
  hasError?: boolean;
}

interface ChatBoxProps {
  message?: string;
}

const ChatBoxWrapper: React.FC<ChatBoxProps> = (props) => {
  const notificationType = 'indicator'
  const removeIndicator = (messages: Message[]) => {
    return filter(messages, (item) => item.type !== notificationType);
  };

  const [disableInput, setDisableInput] = useState<boolean>(false);
  const [disabledInputPlaceholder, setDisabledInputPlaceholder] = useState<string>("Processing, wait a moment...");
  const history = JSON.parse(localStorage.getItem('chatai-messages') || "[]")
  const [messages, setMessages] = useState<Message[]>(removeIndicator(history));
  const [chat, setChat] = useState<webllm.ChatWorkerClient | webllm.ChatModule | null>(null);
  const [question, setQuestion] = useState<string>('');
  const [questionAnswers, setQuestionAnswers] = useState<Record<string, string>>({});

  useEffect(() => {
    const initChatBox = async () => {
      if (window.chat) {
        setChat(window.chat);
      } else {
        const chat = await initChat();
        setChat(chat);
      }
    };

    initChatBox();
  }, []);

  useEffect(() => {
    if (chat) {
      AntdMessage.success('Chat AI is ready.');
      // We must reset the input status after the chat is ready
      setDisableInput(false);
      setDisabledInputPlaceholder("Processing, wait a moment...");

      chat.setInitProgressCallback((report: webllm.InitProgressReport) => {
        setDisableInput(true);
        setDisabledInputPlaceholder(report.text);
      });
    } else {
      setDisableInput(true);
      setDisabledInputPlaceholder("Chat AI is not ready, please wait..")
    }
  }, [chat]);

  const findLastMatchedMessage = (key: string, messages: Message[]): Message | null => {
    for (let i = messages.length - 1; i >= 0; i--) {
      if (messages[i].key === key) {
        return messages[i];
      }
    }

    return null;
  };

  const publishNotification = (key: string, message: string, messages: Message[]): Message[] => {
    const matchedMessage = findLastMatchedMessage(key, messages);

    if (matchedMessage) {
      const newMsg = set(matchedMessage, 'text', message);
      const newMessages = [...messages];
      newMessages[messages.indexOf(matchedMessage)] = newMsg;
      return newMessages;
    } else {
      const notification = {
        key: key,
        author: {
          id: 2,
          username: 'ChatAI',
          avatarUrl: '/assets/images/ai.svg',
        },
        text: message,
        timestamp: +new Date(),
        type: notificationType
      };
      const newMessages = [...messages, notification];
      return newMessages;
    }
  };

  const publishMessage = (key: string, message: string, messages: Message[]) => {
    const matchedMessage = findLastMatchedMessage(key, messages);

    if (matchedMessage) {
      const newMsg = set(matchedMessage, 'text', message);
      const newMessages = [...messages];
      newMessages[messages.indexOf(matchedMessage)] = newMsg;
      return newMessages;
    } else {
      const newMessage = {
        author: {
          id: 2,
          username: 'ChatAI',
          avatarUrl: '/assets/images/ai.svg',
        },
        text: message,
        timestamp: +new Date(),
        type: 'text'
      };
      return [...messages, newMessage];
    }
  };

  const publishMarkdownMessage = (key: string, message: string, messages: Message[]) => {
    const matchedMessage = findLastMatchedMessage(key, messages);

    if (matchedMessage) {
      const newMsg = set(matchedMessage, 'text', message);
      const newMessages = [...messages];
      newMessages[messages.indexOf(matchedMessage)] = newMsg;
      return newMessages;
    } else {
      const newMessage = {
        key: key,
        author: {
          id: 2,
          username: 'ChatAI',
          avatarUrl: '/assets/images/ai.svg',
        },
        text: message,
        timestamp: +new Date(),
        type: 'markdown'
      };
      return [...messages, newMessage];
    }
  };

  const publishErrorMessage = (messages: Message[]) => {
    const newMessage = {
      author: {
        id: 2,
        username: 'ChatAI',
        avatarUrl: '/assets/images/ai.svg',
      },
      text: 'Sorry, error occurred, please try again later.',
      timestamp: +new Date(),
      type: 'text',
    };
    return [...messages, newMessage];
  }

  const string2base64 = (utf8String: string) => {
    // Encode the string into UTF-8 using a TextEncoder
    const encoder = new TextEncoder();
    const encoded = encoder.encode(utf8String);

    // Convert the Uint8Array to a string using built-in btoa function
    const binaryString = String.fromCharCode(...encoded);
    return btoa(binaryString);
  }

  const webLLMPredict = async (question: string, messages: Message[]) => {
    const generateProgressCallback = (_step: number, message: string) => {
      console.log("generateProgressCallback: ", message);
      let length = messages.length;
      let base64string = string2base64(`${question}-${length}`);
      setQuestion(base64string);
      setQuestionAnswers({ ...questionAnswers, [base64string]: message });
    };

    let newMessages = publishNotification('notification', 'Predicting, wait a moment (it\'s slow, be patient)...', messages);
    setDisableInput(true);
    setMessages(newMessages);

    if (!chat) {
      AntdMessage.error('Chat AI is not ready, please try again later.');

      return;
    };

    // Drop the messages that are not from the user
    const filteredMessages = messages.filter((item) => item.author?.id !== 1);
    // Take the first 5 messages as context
    const context = filteredMessages.slice(-5).map((item) => item.text).join('\n');

    const prompt = `
    Context:
    """
    <s> ${context} </s>
    """

    The above is the context, please use the context to answer the following question if the context is relevant. If the context is not relevant, please ignore the context and answer the question directly.

    Question:
    <s> ${question} </s>
    `

    const reply0 = await chat.generate(prompt, generateProgressCallback);
    // The replying is done, we can enable the input again
    setDisableInput(false);
    console.log("reply0: ", reply0);

    console.log(await chat.runtimeStatsText());
  }

  const handleOnSendMessage = (message: string) => {
    const newMessage = {
      author: {
        id: 1,
        username: 'Me',
        avatarUrl: '/assets/images/general.svg',
      },
      text: message,
      timestamp: +new Date(),
      type: 'text'
    };

    const updatedMessages = [...messages, newMessage];
    setMessages(updatedMessages);
    // predict(message, updatedMessages);
    webLLMPredict(message, updatedMessages);
  };

  useEffect(() => {
    localStorage.setItem('chatai-messages', JSON.stringify(messages));
  }, [messages]);

  useEffect(() => {
    // How to know if the message contains any non-english characters?

    if (props.message) {
      if (disableInput === false) {
        handleOnSendMessage(props.message);
      } else {
        console.log('Chat AI is processing, please wait...');
        AntdMessage.warning('Chat AI is processing, please wait...', 5)
      }
    }
  }, [props.message]);

  useEffect(() => {
    if (question && questionAnswers[question]) {
      const newMessages = publishMarkdownMessage(question, questionAnswers[question], messages);
      setMessages(removeIndicator(newMessages));
    }
  }, [questionAnswers]);

  return <ReactChatPlugin
    messages={messages}
    userId={1}
    disableInput={disableInput}
    disabledInputPlaceholder={disabledInputPlaceholder}
    showTypingIndicator={true}
    onSendMessage={handleOnSendMessage}
    clearHistory={() => {
      setMessages([]);
      localStorage.setItem('chatai-messages', JSON.stringify([]));
    }}
    width={'100%'}
    height={'calc(100vh - 58px)'}
    onSendKey={"shiftKey"}
    rehypePlugins={[rehypeRaw, rehypeHighlight]}
    deleteHandler={(message: Message) => {
      console.log('deleteHandler: ', message);
      const newMessages = messages.filter((item) => item.timestamp !== message.timestamp);
      setMessages(newMessages);
      localStorage.setItem('chatai-messages', JSON.stringify(newMessages));
    }}
  />;
}

export default ChatBoxWrapper;
