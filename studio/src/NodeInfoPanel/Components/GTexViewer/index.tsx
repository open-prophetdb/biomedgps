import React, { useEffect, useState } from 'react'
import GTexGeneViolinViewer from './Components/GTexGeneViolinViewer';
import GTexTranscriptViewer from './Components/GTexTranscriptViewer';

import './index.less';

type GTexViewerProps = {
  rootId?: string,
  type: string, // gene or transcript
  title?: string,
  ensemblId: string, // e.g. 'ENSG00000141510'
  description?: string,
}

const links = [
  {
    rel: "stylesheet",
    href: "https://gtexportal.org/external/bootstrap/3.3.7/bootstrap.min.css"
  },
  {
    rel: "stylesheet",
    href: "https://gtexportal.org/external/jquery-ui-1.11.4.custom/jquery-ui.css"
  },
  {
    rel: "stylesheet",
    href: "https://use.fontawesome.com/releases/v5.5.0/css/all.css"
  }
]

const loadStyles = (links: any) => {
  links.forEach((link: any) => {
    if (!document.querySelector(`link[href="${link.href}"]`)) {
      const linkElement = document.createElement('link');
      linkElement.rel = link.rel;
      linkElement.href = link.href;
      document.head.appendChild(linkElement);
    }
  });
}

const unloadStyles = (links: any) => {
  links.forEach((link: any) => {
    const existingLinkElement = document.querySelector(`link[href="${link.href}"]`);
    if (existingLinkElement) {
      document.head.removeChild(existingLinkElement);
    }
  });
}

const GTexViewer: React.FC<GTexViewerProps> = (props) => {
  const [rootId, setRootId] = useState<string>("");
  const [versionedEnsemblId, setVersionedEnsemblId] = useState<string>("");

  const fetchVersionedEnsemblId = async (ensemblId: string) => {
    const response = await fetch(`https://gtexportal.org/api/v2/reference/geneSearch?geneId=${ensemblId}`);
    const data = await response.json();
    if (data.data.length > 0) {
      return data.data[0].gencodeId;
    } else {
      return null;
    }
  }

  useEffect(() => {
    if (!props.rootId) {
      setRootId('gtex-viewer')
    } else {
      setRootId(props.rootId)
    }

    loadStyles(links);

    return () => {
      unloadStyles(links);
    };
  }, []);

  useEffect(() => {
    const init = async () => {
      const versionedEnsemblId = await fetchVersionedEnsemblId(props.ensemblId);
      setVersionedEnsemblId(versionedEnsemblId);
    }

    init();
  }, [props.ensemblId]);

  return (
    <div className='gtex-viewer'>
      <div className='summary'>
        <h3 className='summary-title'>Description</h3>
        <p className='summary-content'>
          {props.description || 'Gene expression data from the Genotype-Tissue Expression (GTEx) project.'}
        </p>
      </div>
      {
        props.type == 'transcript' ?
          <div className='transcript-figures'>
            <GTexTranscriptViewer
              rootId={rootId + '-isoform-transposed'}
              type="isoformTransposed" title={props.title || "Isoform Transposed"}
              geneId={versionedEnsemblId} />
            <GTexTranscriptViewer
              rootId={rootId + '-exon'} title={props.title || "Exon"}
              type="exon" geneId={versionedEnsemblId} />
            <GTexTranscriptViewer
              rootId={rootId + '-junction'} title={props.title || "Junction"}
              type="junction" geneId={versionedEnsemblId} />
          </div>
          : null
      }
      {
        props.type == 'gene' ?
          <div className='gene-figures'>
            <GTexGeneViolinViewer title={props.title || 'Violin Plot'}
              geneId={versionedEnsemblId} />
          </div>
          : null
      }
    </div>
  )
}

export default GTexViewer;