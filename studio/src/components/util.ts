export const getJwtAccessToken = (): string | null => {
    let jwtToken = null;
    // Check if the cookie exists
    if (document.cookie && document.cookie.includes("jwt_access_token=")) {
        // Retrieve the cookie value
        // @ts-ignore
        jwtToken = document.cookie
            .split("; ")
            .find((row) => row.startsWith("jwt_access_token="))
            .split("=")[1];
    }

    if (jwtToken) {
        console.log("JWT access token found in the cookie.");
        return jwtToken;
    } else {
        console.log("JWT access token not found in the cookie.");
        return null;
    }
}

export const guessColor = (text: string): string => {
    const colors: Record<string, string> = {
        Anatomy: "#1f78b4",
        BiologicalProcess: "#b15928",
        CellularComponent: "#cab2d6",
        Compound: "#33a02c",
        Disease: "#fb9a99",
        Gene: "#e31a1c",
        Metabolite: "#a6cee3",
        MolecularFunction: "#b2df8a",
        Pathway: "#ff7f00",
        PharmacologicClass: "#fdbf6f",
        SideEffect: "#ffff99",
        Symptom: "#6a3d9a"
    }

    return colors[text] || "#108ee9";
}

export const expectedOrder: string[] = [
    '9606',
    '10090',
    '10116',
    '9541',
    '9544',
    '9598'
]

export const expectedSpecies: Record<string, string[]> = {
    '9606': ['Human', 'Human'],
    '10090': ['Mouse', 'Mouse'],
    '10116': ['Rattus norvegicus', 'Rat'],
    '9541': ['Macaca fascicularis', 'M.fascicularis'],
    '9544': ['Macaca mulatta', 'M.mulatta'],
    '9598': ['Chimpanzee', 'Chimpanzee']
}

export const guessSpecies = (taxid: string) => {
    return expectedSpecies[`${taxid}`] ? expectedSpecies[`${taxid}`][0] : 'Unknown'
}

export const guessSpeciesAbbr = (taxid: string) => {
    return expectedSpecies[`${taxid}`] ? expectedSpecies[`${taxid}`][1] : 'Unknown'
}

export const isExpectedSpecies = (taxid: string) => {
    return expectedSpecies[`${taxid}`] ? true : false
}
