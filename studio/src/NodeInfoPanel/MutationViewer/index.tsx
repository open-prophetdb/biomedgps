import React, { Component } from 'react'
import LollipopPlot from 'react-mutation-plot';

export const lollipops = [
  {
    'codon': 858,
    'count': 23,
    'tooltip': {
      'header': 'Title',
      'body': 'Description'
    },
    'color': '#008000',
    'id': 'variant-id-001',
    'label': {
      'text': 'L858R',
      'textAnchor': 'middle',
      'fontSize': 10,
      'fontFamily': 'arial'
    },
    'selected': true
  },
  {
    'codon': 746,
    'count': 17,
    'tooltip': {
      'header': 'Title',
      'body': 'Description'
    },
    'color': '#993404',
    'selected': true
  },
  {
    'codon': 861,
    'count': 5,
    'tooltip': {
      'header': 'Title',
      'body': 'Description'
    },
    'color': '#008000'
  },
  {
    'codon': 747,
    'count': 5,
    'tooltip': {
      'header': 'Title',
      'body': 'Description'
    },
    'color': '#993404',
    'selected': false
  },
  {
    'codon': 768,
    'count': 3,
    'tooltip': {
      'header': 'Title',
      'body': 'Description'
    },
    'color': '#008000'
  },
  {
    'codon': 754,
    'count': 3,
    'tooltip': {
      'header': 'Title',
      'body': 'Description'
    },
    'color': '#008000'
  },
  {
    'codon': 719,
    'count': 3,
    'tooltip': {
      'header': 'Title',
      'body': 'Description'
    },
    'color': '#008000'
  },
  {
    'codon': 709,
    'count': 3,
    'tooltip': {
      'header': 'Title',
      'body': 'Description'
    },
    'color': '#993404'
  },
  {
    'codon': 833,
    'count': 2,
    'tooltip': {
      'header': 'Title',
      'body': 'Description'
    },
    'color': '#008000'
  },
  {
    'codon': 1,
    'count': 1,
    'tooltip': {
      'header': 'Title',
      'body': 'Description'
    },
    'color': '#cf58bc'
  }
]

export const domains = [
  {
    'startCodon': 57,
    'endCodon': 167,
    'label': 'Recep_L_domain',
    'color': '#2dcf00',
    'tooltip': {
      'header': 'Recep_L_domain',
      'body': 'Recep_L_domain (57 - 167)'
    }
  },
  {
    'startCodon': 185,
    'endCodon': 338,
    'label': 'Furin-like',
    'color': '#ff5353',
    'tooltip': {
      'header': 'Furin-like'
    }
  },
  {
    'startCodon': 361,
    'endCodon': 480,
    'label': 'Recep_L_domain',
    'color': '#2dcf00'
  },
  {
    'startCodon': 505,
    'endCodon': 636,
    'label': 'GF_recep_IV',
    'color': '#5b5bff',
    'tooltip': {
      'header': 'Title',
      'body': 'Description'
    }
  },
  {
    'startCodon': 713,
    'endCodon': 965,
    'label': 'Pkinase_Tyr',
    'color': '#ebd61d',
    'tooltip': {
      'header': 'Title',
      'body': 'Description'
    }
  }
]

class MutationViewer extends Component {
  render() {
    return (
      <LollipopPlot
        domains={domains}
        lollipops={lollipops}
        vizWidth={1000}
        vizHeight={300}
        hugoGeneSymbol={'PRG4'}
        xMax={1000}
        yMax={300}
        onLollipopClick={() => { }}
      />
    )
  }
}

export default MutationViewer;