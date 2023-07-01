export const play_ = (id) => () => document.getElementById(id).play()

export const pause_ = (id) => () => document.getElementById(id).pause()

export const playerState_ = (id) => () => {
  const video = document.getElementById(id)

  if (video && video.children[0] && video.children[0].src && video.duration && video.currentTime) {
    const url = video.children[0].src

    return {
      path: url.substr(url.lastIndexOf("/play/") + 6),
      duration: video.duration,
      time: video.currentTime
    }
  } else {
    return null
  }
}

export const setCurrentTime_ = (id) => (time) => () => {
  const video = document.getElementById(id)

  if (video && video.currentTime) {
    video.currentTime = time
  }
}
