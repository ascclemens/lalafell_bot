convertTimestamps = (className) ->
  elements = document.getElementsByClassName className
  for element in elements
    do (element) ->
      timestamp = Number(element.getAttribute('data-timestamp')) * 1000
      element.setAttribute 'title', element.innerHTML + ' UTC'
      element.innerHTML = moment(timestamp).format 'MM/DD/YYYY [at] HH:mm:ss'
