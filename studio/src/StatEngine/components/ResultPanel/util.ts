export const formatDuration = (duration: number) => {
    const durationInSeconds = duration / 1000;

    if (durationInSeconds <= 0) {
        return '0s';
    }

    if (durationInSeconds < 60) {
        return `${Math.round(durationInSeconds)}s`;
    } else if (durationInSeconds < 3600) {
        const minutes = Math.floor(durationInSeconds / 60);
        const seconds = Math.round(durationInSeconds % 60);
        return seconds > 0 ? `${minutes}min ${seconds}s` : `${minutes}min`;
    } else {
        const hours = Math.floor(durationInSeconds / 3600);
        const minutes = Math.floor((durationInSeconds % 3600) / 60);
        return minutes > 0 ? `${hours}h ${minutes}min` : `${hours}h`;
    }
};
