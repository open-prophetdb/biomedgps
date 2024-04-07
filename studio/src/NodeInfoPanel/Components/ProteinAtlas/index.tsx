import React, { useEffect, useState, useRef } from 'react'
import { Spin } from 'antd'

type ProteinAtlasProps = {
  rootId?: string,
  ensemblId: string,
  // Only support official gene symbol for now
  geneSymbol: string,
}

const ProteinAtlas: React.FC<ProteinAtlasProps> = (props) => {
  const [rootId, setRootId] = useState<string>("");
  const [src, setSrc] = useState<string>("");
  const ref = useRef(null);
  const [loading, setLoading] = useState<boolean>(true);

  useEffect(() => {
    if (props.geneSymbol && props.ensemblId) {
      let host = window.location.host;
      if (host.startsWith('localhost')) {
        host = "drugs.3steps.cn"
      }

      setSrc(`https://${host}/proxy/protein_atlas/${props.ensemblId}-${props.geneSymbol}`)
    }
  }, [props.geneSymbol, props.ensemblId]);

  useEffect(() => {
    if (!props.rootId) {
      setRootId('protein-atlas-iframe')
    } else {
      setRootId(props.rootId)
    }
  }, []);

  return (
    <div id="iframe-container" style={{ position: 'relative', width: '100%', height: '100%' }}>
      <iframe id={rootId} title="Protein Atlas" src={src}
        style={{ width: '100%', height: '100%', border: 'none', minHeight: '1000px' }} onLoad={() => {
          setLoading(false)
        }} ref={ref} />
      {
        loading ? <Spin spinning={loading} style={{
          position: 'absolute', top: 0, left: 0, width: '100%', height: '100%', minHeight: '1000px'
        }}></Spin> : null
      }
    </div>
  )
}

export default ProteinAtlas;