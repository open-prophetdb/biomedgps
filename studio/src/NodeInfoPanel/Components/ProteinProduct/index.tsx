import React, { useEffect, useState, useRef } from 'react'
import { Spin } from 'antd'

import './index.less'

type ProteinAtlasProps = {
  rootId?: string,
  uniprotId: string,
}

const ProteinProduct: React.FC<ProteinAtlasProps> = (props) => {
  const [rootId, setRootId] = useState<string>("");
  const [src, setSrc] = useState<string>("");
  const ref = useRef(null);
  const [loading, setLoading] = useState<boolean>(true);

  useEffect(() => {
    if (props.uniprotId) {
      let host = window.location.host;
      if (host.startsWith('localhost')) {
        host = "drugs.3steps.cn"
      }

      setSrc(`https://${host}/proxy/rndsystems/cn/search?keywords=${props.uniprotId}&numResults=100`)
    }
  }, [props.uniprotId]);

  useEffect(() => {
    if (!props.rootId) {
      setRootId('protein-atlas-iframe')
    } else {
      setRootId(props.rootId)
    }

    window.addEventListener('message', function (e) {
      if (e.data.type === 'linkClicked') {
        setLoading(true)
      }
    });

    return () => {
      window.removeEventListener('message', function (e) {
        if (e.data.type === 'linkClicked') {
          setLoading(true)
        }
      });
    }
  }, []);

  return (
    <div className="iframe-container" style={{ position: 'relative', width: '100%', height: '100%' }}>
      <iframe id={rootId} title="R&D Systems" src={src} className={loading ? 'hidden' : ''}
        style={{ width: '100%', border: 'none', height: 'calc(100vh - 120px)' }} onLoad={() => {
          setLoading(false)
        }} ref={ref} />
      {
        loading ? <Spin spinning={loading} style={{
          position: 'absolute', top: 0, left: 0, width: '100%', height: 'calc(100vh - 120px)', minHeight: '1000px'
        }}></Spin> : null
      }
    </div>
  )
}

export default ProteinProduct;