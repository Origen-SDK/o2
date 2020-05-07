
$(function() {
  // Due to dynamic loading of the submenus in the 'Pages' heading, straight CSS is dumped down the order
  // and gets overriden by some JS.
  // Its possible to extend that specific class, but then the values need to be hard-coded, but it'd be
  // more preferable if we can instead inherit from a couple levels up
  // This'll still work as expected even if the theme is changed or if custom CSS to change some
  // of the colors around is brought in.
  $('#navbar-pages > ul')
    .children('li')
    .find('ul')
    .css(
      "background-color",
      $('#navbar-pages > ul').css("background-color")
    );
});
